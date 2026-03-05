/// Detect database driver from connection string.
pub fn detect_driver(connection_string: &str) -> Option<String> {
    let lower = connection_string.to_lowercase();

    match lower.as_str() {
        s if s.starts_with("postgresql://") || s.starts_with("postgres://") => {
            Some("postgres".to_string())
        }
        s if s.starts_with("mysql://") => Some("mysql".to_string()),
        s if s.starts_with("sqlite://") || s.starts_with("file:") => {
            Some("sqlite".to_string())
        }
        s if s.starts_with("mssql://") || s.starts_with("sqlserver://") => {
            Some("mssql".to_string())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_postgres() {
        assert_eq!(
            detect_driver("postgresql://user:pass@localhost/db"),
            Some("postgres".to_string())
        );
        assert_eq!(
            detect_driver("postgres://user:pass@localhost/db"),
            Some("postgres".to_string())
        );
    }

    #[test]
    fn test_detect_mysql() {
        assert_eq!(
            detect_driver("mysql://user:pass@localhost/db"),
            Some("mysql".to_string())
        );
    }

    #[test]
    fn test_detect_sqlite() {
        assert_eq!(
            detect_driver("sqlite://path/to/db.sqlite"),
            Some("sqlite".to_string())
        );
    }

    #[test]
    fn test_detect_mssql() {
        assert_eq!(
            detect_driver("mssql://server/database"),
            Some("mssql".to_string())
        );
        assert_eq!(
            detect_driver("sqlserver://server/database"),
            Some("mssql".to_string())
        );
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(detect_driver("unknown://something"), None);
    }
}
