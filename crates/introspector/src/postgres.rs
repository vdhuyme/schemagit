use crate::{Introspector, IntrospectorError, IntrospectorResult};
use schemagit_core::{Column, DatabaseSchema, ForeignKey, Index, Table};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres, Row};

/// PostgreSQL schema introspector implementation.
pub struct PostgresIntrospector {
    connection_string: String,
    pool: Option<Pool<Postgres>>,
}

impl PostgresIntrospector {
    /// Create a new PostgreSQL introspector with the given connection string.
    pub fn new(connection_string: String) -> Self {
        Self {
            connection_string,
            pool: None,
        }
    }

    /// Get or create a connection pool.
    async fn get_pool(&mut self) -> IntrospectorResult<&Pool<Postgres>> {
        if self.pool.is_none() {
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&self.connection_string)
                .await
                .map_err(|e| {
                    IntrospectorError::ConnectionError(e.to_string())
                })?;
            self.pool = Some(pool);
        }
        Ok(self.pool.as_ref().unwrap())
    }

    /// Introspect tables from the database.
    async fn introspect_tables(&mut self) -> IntrospectorResult<Vec<String>> {
        let pool = self.get_pool().await?;

        let rows = sqlx::query(
            r#"
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = 'public'
              AND table_type = 'BASE TABLE'
            ORDER BY table_name
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| IntrospectorError::QueryError(e.to_string()))?;

        Ok(rows.iter().map(|row| row.get("table_name")).collect())
    }

    /// Introspect columns for a specific table.
    async fn introspect_columns(
        &mut self,
        table_name: &str,
    ) -> IntrospectorResult<Vec<Column>> {
        let pool = self.get_pool().await?;

        let rows = sqlx::query(
            r#"
            SELECT 
                column_name,
                data_type,
                is_nullable,
                column_default
            FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = $1
            ORDER BY ordinal_position
            "#,
        )
        .bind(table_name)
        .fetch_all(pool)
        .await
        .map_err(|e| IntrospectorError::QueryError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|row| {
                let nullable: String = row.get("is_nullable");
                Column {
                    name: row.get("column_name"),
                    data_type: row.get("data_type"),
                    nullable: nullable == "YES",
                    default: row.get("column_default"),
                }
            })
            .collect())
    }

    /// Introspect indexes for a specific table.
    async fn introspect_indexes(
        &mut self,
        table_name: &str,
    ) -> IntrospectorResult<Vec<Index>> {
        let pool = self.get_pool().await?;

        let rows = sqlx::query(
            r#"
            SELECT 
                i.relname as index_name,
                a.attname as column_name,
                ix.indisunique as is_unique
            FROM pg_class t
            JOIN pg_index ix ON t.oid = ix.indrelid
            JOIN pg_class i ON i.oid = ix.indexrelid
            JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey)
            WHERE t.relname = $1
              AND t.relkind = 'r'
              AND i.relname NOT LIKE '%_pkey'
            ORDER BY i.relname, a.attnum
            "#,
        )
        .bind(table_name)
        .fetch_all(pool)
        .await
        .map_err(|e| IntrospectorError::QueryError(e.to_string()))?;

        // Group columns by index name
        let mut indexes_map: std::collections::HashMap<
            String,
            (Vec<String>, bool),
        > = std::collections::HashMap::new();

        for row in rows {
            let index_name: String = row.get("index_name");
            let column_name: String = row.get("column_name");
            let is_unique: bool = row.get("is_unique");

            indexes_map
                .entry(index_name)
                .or_insert_with(|| (Vec::new(), is_unique))
                .0
                .push(column_name);
        }

        Ok(indexes_map
            .into_iter()
            .map(|(name, (columns, unique))| Index {
                name,
                columns,
                unique,
            })
            .collect())
    }

    /// Introspect foreign keys for a specific table.
    async fn introspect_foreign_keys(
        &mut self,
        table_name: &str,
    ) -> IntrospectorResult<Vec<ForeignKey>> {
        let pool = self.get_pool().await?;

        let rows = sqlx::query(
            r#"
            SELECT
                tc.constraint_name,
                kcu.column_name,
                ccu.table_name AS foreign_table_name,
                ccu.column_name AS foreign_column_name
            FROM information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
              ON tc.constraint_name = kcu.constraint_name
              AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage AS ccu
              ON ccu.constraint_name = tc.constraint_name
              AND ccu.table_schema = tc.table_schema
            WHERE tc.constraint_type = 'FOREIGN KEY'
              AND tc.table_name = $1
              AND tc.table_schema = 'public'
            "#,
        )
        .bind(table_name)
        .fetch_all(pool)
        .await
        .map_err(|e| IntrospectorError::QueryError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|row| ForeignKey {
                name: row.get("constraint_name"),
                column: row.get("column_name"),
                ref_table: row.get("foreign_table_name"),
                ref_column: row.get("foreign_column_name"),
            })
            .collect())
    }
}

#[async_trait::async_trait]
impl Introspector for PostgresIntrospector {
    async fn introspect_schema(&self) -> IntrospectorResult<DatabaseSchema> {
        // Clone self to get mutable access
        let mut introspector = PostgresIntrospector {
            connection_string: self.connection_string.clone(),
            pool: None,
        };

        let table_names = introspector.introspect_tables().await?;

        let mut tables = Vec::new();
        for table_name in table_names {
            let columns = introspector.introspect_columns(&table_name).await?;
            let indexes = introspector.introspect_indexes(&table_name).await?;
            let foreign_keys =
                introspector.introspect_foreign_keys(&table_name).await?;

            tables.push(Table {
                name: table_name,
                columns,
                indexes,
                foreign_keys,
            });
        }

        Ok(DatabaseSchema { tables })
    }

    async fn database_name(&self) -> IntrospectorResult<String> {
        // Extract database name from connection string
        let parts: Vec<&str> = self.connection_string.split('/').collect();
        let db_with_params = parts.last().ok_or_else(|| {
            IntrospectorError::ConnectionError(
                "Invalid connection string".to_string(),
            )
        })?;

        let db_name =
            db_with_params.split('?').next().unwrap_or(db_with_params);
        Ok(db_name.to_string())
    }

    fn database_type(&self) -> &str {
        "postgres"
    }
}
