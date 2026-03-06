use crate::{Introspector, IntrospectorError, IntrospectorResult};
use percent_encoding::percent_decode_str;
use schemagit_core::{Column, DatabaseSchema, ForeignKey, Index, Table};
use tiberius::{AuthMethod, Client, Config, EncryptionLevel, Row};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;
use url::Url;

const NULLABLE_YES: &str = "YES";

/// SQL Server (MSSQL) schema introspector implementation.
pub struct MssqlIntrospector {
    connection_string: String,
    client: Option<Client<tokio_util::compat::Compat<TcpStream>>>,
}

impl MssqlIntrospector {
    /// Create a new MSSQL introspector with the given connection string.
    pub fn new(connection_string: String) -> Self {
        Self {
            connection_string,
            client: None,
        }
    }

    async fn get_client(
        &mut self,
    ) -> IntrospectorResult<&mut Client<tokio_util::compat::Compat<TcpStream>>>
    {
        if self.client.is_none() {
            let config = parse_mssql_url(&self.connection_string)?;
            let addr = config.get_addr();

            let tcp = TcpStream::connect(addr).await.map_err(|e| {
                IntrospectorError::ConnectionError(e.to_string())
            })?;
            tcp.set_nodelay(true).map_err(|e| {
                IntrospectorError::ConnectionError(e.to_string())
            })?;

            let client =
                Client::connect(config, tcp.compat_write()).await.map_err(
                    |e| IntrospectorError::ConnectionError(e.to_string()),
                )?;

            self.client = Some(client);
        }

        self.client.as_mut().ok_or_else(|| {
            IntrospectorError::ConnectionError(
                "MSSQL client is not initialized".to_string(),
            )
        })
    }

    async fn introspect_tables(&mut self) -> IntrospectorResult<Vec<String>> {
        let client = self.get_client().await?;

        let rows: Vec<Row> = client
            .query(
                r#"
                SELECT TABLE_NAME AS table_name
                FROM INFORMATION_SCHEMA.TABLES
                WHERE TABLE_SCHEMA = 'dbo'
                AND TABLE_TYPE = 'BASE TABLE'
                ORDER BY TABLE_NAME
                "#,
                &[],
            )
            .await
            .map_err(|e| IntrospectorError::QueryError(e.to_string()))?
            .into_first_result()
            .await
            .map_err(|e| IntrospectorError::QueryError(e.to_string()))?;

        Ok(rows
            .iter()
            .filter_map(|row| {
                row.get::<&str, _>("table_name").map(|s| s.to_string())
            })
            .collect())
    }

    async fn introspect_columns(
        &mut self,
        table_name: &str,
    ) -> IntrospectorResult<Vec<Column>> {
        let client = self.get_client().await?;

        let rows: Vec<Row> = client
            .query(
                r#"
                SELECT
                    COLUMN_NAME AS column_name,
                    DATA_TYPE AS data_type,
                    IS_NULLABLE AS is_nullable,
                    COLUMN_DEFAULT AS column_default,
                    CHARACTER_MAXIMUM_LENGTH AS char_max_len,
                    NUMERIC_PRECISION AS numeric_precision,
                    NUMERIC_SCALE AS numeric_scale
                FROM INFORMATION_SCHEMA.COLUMNS
                WHERE TABLE_SCHEMA = 'dbo'
                AND TABLE_NAME = @P1
                ORDER BY ORDINAL_POSITION
                "#,
                &[&table_name],
            )
            .await
            .map_err(|e| IntrospectorError::QueryError(e.to_string()))?
            .into_first_result()
            .await
            .map_err(|e| IntrospectorError::QueryError(e.to_string()))?;

        Ok(rows.iter().map(Self::column_from_row).collect())
    }

    async fn introspect_indexes(
        &mut self,
        table_name: &str,
    ) -> IntrospectorResult<Vec<Index>> {
        let client = self.get_client().await?;

        let rows: Vec<Row> = client
            .query(
                r#"
                SELECT
                    i.name AS index_name,
                    c.name AS column_name,
                    i.is_unique AS is_unique,
                    ic.key_ordinal AS key_ordinal
                FROM sys.indexes i
                INNER JOIN sys.index_columns ic
                    ON i.object_id = ic.object_id
                AND i.index_id = ic.index_id
                INNER JOIN sys.columns c
                    ON ic.object_id = c.object_id
                AND ic.column_id = c.column_id
                INNER JOIN sys.tables t
                    ON i.object_id = t.object_id
                INNER JOIN sys.schemas s
                    ON t.schema_id = s.schema_id
                WHERE t.name = @P1
                AND s.name = 'dbo'
                AND i.is_primary_key = 0
                AND i.is_unique_constraint = 0
                AND ic.key_ordinal > 0
                ORDER BY i.name, ic.key_ordinal
                "#,
                &[&table_name],
            )
            .await
            .map_err(|e| IntrospectorError::QueryError(e.to_string()))?
            .into_first_result()
            .await
            .map_err(|e| IntrospectorError::QueryError(e.to_string()))?;

        let mut indexes_map: std::collections::HashMap<
            String,
            (Vec<String>, bool),
        > = std::collections::HashMap::new();

        for row in rows {
            let index_name: Option<&str> = row.get("index_name");
            let column_name: Option<&str> = row.get("column_name");
            let is_unique: Option<bool> = row.get("is_unique");

            let (Some(index_name), Some(column_name), Some(is_unique)) =
                (index_name, column_name, is_unique)
            else {
                continue;
            };

            indexes_map
                .entry(index_name.to_string())
                .or_insert_with(|| (Vec::new(), is_unique))
                .0
                .push(column_name.to_string());
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

    async fn introspect_foreign_keys(
        &mut self,
        table_name: &str,
    ) -> IntrospectorResult<Vec<ForeignKey>> {
        let client = self.get_client().await?;

        let rows: Vec<Row> = client
            .query(
                r#"
                SELECT
                    fk.name AS constraint_name,
                    pc.name AS column_name,
                    rt.name AS foreign_table_name,
                    rc.name AS foreign_column_name
                FROM sys.foreign_keys fk
                INNER JOIN sys.foreign_key_columns fkc
                    ON fk.object_id = fkc.constraint_object_id
                INNER JOIN sys.tables pt
                    ON fkc.parent_object_id = pt.object_id
                INNER JOIN sys.schemas ps
                    ON pt.schema_id = ps.schema_id
                INNER JOIN sys.columns pc
                    ON pc.object_id = pt.object_id
                AND pc.column_id = fkc.parent_column_id
                INNER JOIN sys.tables rt
                    ON fkc.referenced_object_id = rt.object_id
                INNER JOIN sys.columns rc
                    ON rc.object_id = rt.object_id
                AND rc.column_id = fkc.referenced_column_id
                WHERE pt.name = @P1
                AND ps.name = 'dbo'
                ORDER BY fk.name, fkc.constraint_column_id
                "#,
                &[&table_name],
            )
            .await
            .map_err(|e| IntrospectorError::QueryError(e.to_string()))?
            .into_first_result()
            .await
            .map_err(|e| IntrospectorError::QueryError(e.to_string()))?;

        Ok(rows.iter().map(Self::foreign_key_from_row).collect())
    }

    fn column_from_row(row: &Row) -> Column {
        let nullable: Option<&str> = row.get("is_nullable");
        let data_type: Option<&str> = row.get("data_type");

        // INFORMATION_SCHEMA returns these with MSSQL-native types that don't
        // always map to i32, so we coerce common representations.
        let char_max_len = Self::coerce_optional_i32(row, "char_max_len");
        let numeric_precision =
            Self::coerce_optional_i32(row, "numeric_precision");
        let numeric_scale = Self::coerce_optional_i32(row, "numeric_scale");

        Column {
            name: row
                .get::<&str, _>("column_name")
                .unwrap_or_default()
                .to_string(),
            data_type: format_mssql_type(
                data_type.unwrap_or_default(),
                char_max_len,
                numeric_precision,
                numeric_scale,
            ),
            nullable: nullable == Some(NULLABLE_YES),
            default: row.get::<&str, _>("column_default").map(str::to_string),
        }
    }

    fn foreign_key_from_row(row: &Row) -> ForeignKey {
        ForeignKey {
            name: row
                .get::<&str, _>("constraint_name")
                .unwrap_or_default()
                .to_string(),
            column: row
                .get::<&str, _>("column_name")
                .unwrap_or_default()
                .to_string(),
            ref_table: row
                .get::<&str, _>("foreign_table_name")
                .unwrap_or_default()
                .to_string(),
            ref_column: row
                .get::<&str, _>("foreign_column_name")
                .unwrap_or_default()
                .to_string(),
        }
    }

    fn coerce_optional_i32(row: &Row, key: &str) -> Option<i32> {
        row.get::<i32, _>(key)
            .or_else(|| row.get::<u8, _>(key).map(i32::from))
            .or_else(|| row.get::<i64, _>(key).map(|v| v as i32))
    }
}

#[async_trait::async_trait]
impl Introspector for MssqlIntrospector {
    async fn introspect_schema(&self) -> IntrospectorResult<DatabaseSchema> {
        let mut introspector = MssqlIntrospector {
            connection_string: self.connection_string.clone(),
            client: None,
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
        let url = Url::parse(&self.connection_string)
            .map_err(|e| IntrospectorError::ConnectionError(e.to_string()))?;
        let db = url.path().trim_start_matches('/').to_string();
        if db.is_empty() {
            return Err(IntrospectorError::ConnectionError(
                "Missing database name in connection string path".to_string(),
            ));
        }
        Ok(db)
    }

    fn database_type(&self) -> &str {
        "mssql"
    }
}

fn format_mssql_type(
    data_type: &str,
    char_max_len: Option<i32>,
    numeric_precision: Option<i32>,
    numeric_scale: Option<i32>,
) -> String {
    let lower = data_type.to_lowercase();

    match lower.as_str() {
        "varchar" | "nvarchar" | "char" | "nchar" | "binary" | "varbinary" => {
            match char_max_len {
                Some(-1) => format!("{data_type}(max)"),
                Some(n) => format!("{data_type}({n})"),
                None => data_type.to_string(),
            }
        }
        "decimal" | "numeric" => match (numeric_precision, numeric_scale) {
            (Some(p), Some(s)) => format!("{data_type}({p},{s})"),
            (Some(p), None) => format!("{data_type}({p})"),
            _ => data_type.to_string(),
        },
        _ => data_type.to_string(),
    }
}

fn parse_mssql_url(url_str: &str) -> IntrospectorResult<Config> {
    let url = Url::parse(url_str)
        .map_err(|e| IntrospectorError::ConnectionError(e.to_string()))?;

    let scheme = url.scheme().to_lowercase();
    if scheme != "mssql" && scheme != "sqlserver" {
        return Err(IntrospectorError::ConnectionError(format!(
            "Invalid MSSQL URL scheme: {}",
            url.scheme()
        )));
    }

    let host = url.host_str().ok_or_else(|| {
        IntrospectorError::ConnectionError(
            "Missing host in connection string".to_string(),
        )
    })?;

    let port = url.port().unwrap_or(1433);
    let database = url.path().trim_start_matches('/').to_string();
    if database.is_empty() {
        return Err(IntrospectorError::ConnectionError(
            "Missing database name in connection string path".to_string(),
        ));
    }

    let username = url.username();
    if username.is_empty() {
        return Err(IntrospectorError::ConnectionError(
            "Missing username in connection string".to_string(),
        ));
    }
    let password = url.password().ok_or_else(|| {
        IntrospectorError::ConnectionError(
            "Missing password in connection string".to_string(),
        )
    })?;

    // Percent-decode username and password to allow special characters.
    let password = percent_decode_str(password)
        .decode_utf8()
        .map_err(|e| IntrospectorError::ConnectionError(e.to_string()))?
        .to_string();

    let mut config = Config::new();
    config.host(host);
    config.port(port);
    config.authentication(AuthMethod::sql_server(username, password));
    config.database(database);
    config.encryption(EncryptionLevel::Required);
    config.trust_cert();

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_varchar_len() {
        assert_eq!(
            format_mssql_type("varchar", Some(50), None, None),
            "varchar(50)"
        );
    }

    #[test]
    fn formats_nvarchar_max() {
        assert_eq!(
            format_mssql_type("nvarchar", Some(-1), None, None),
            "nvarchar(max)"
        );
    }

    #[test]
    fn formats_decimal_precision_scale() {
        assert_eq!(
            format_mssql_type("decimal", None, Some(18), Some(2)),
            "decimal(18,2)"
        );
    }

    #[test]
    fn parses_mssql_url_basic() {
        let cfg = parse_mssql_url("mssql://sa:pw@localhost:1433/mydb").unwrap();
        let addr = cfg.get_addr();
        assert!(addr.to_string().contains("localhost:1433"));
    }
}
