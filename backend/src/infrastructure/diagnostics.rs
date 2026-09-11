use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DatabaseErrorCategory {
    Connection,
    PoolTimeout,
    DatabaseRejection,
    Protocol,
    Decode,
    Transaction,
    Unknown,
}

impl fmt::Display for DatabaseErrorCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Connection => "connection",
            Self::PoolTimeout => "pool_timeout",
            Self::DatabaseRejection => "database_rejection",
            Self::Protocol => "protocol",
            Self::Decode => "decode",
            Self::Transaction => "transaction",
            Self::Unknown => "unknown",
        };
        formatter.write_str(value)
    }
}

pub fn classify_sqlx_error(error: &sqlx::Error) -> DatabaseErrorCategory {
    match error {
        sqlx::Error::Configuration(_)
        | sqlx::Error::Io(_)
        | sqlx::Error::Tls(_)
        | sqlx::Error::PoolClosed => DatabaseErrorCategory::Connection,
        sqlx::Error::PoolTimedOut => DatabaseErrorCategory::PoolTimeout,
        sqlx::Error::Database(_) => DatabaseErrorCategory::DatabaseRejection,
        sqlx::Error::Protocol(_) => DatabaseErrorCategory::Protocol,
        sqlx::Error::TypeNotFound { .. }
        | sqlx::Error::ColumnIndexOutOfBounds { .. }
        | sqlx::Error::ColumnNotFound(_)
        | sqlx::Error::ColumnDecode { .. }
        | sqlx::Error::Decode(_) => DatabaseErrorCategory::Decode,
        sqlx::Error::Migrate(_)
        | sqlx::Error::InvalidSavePointStatement
        | sqlx::Error::BeginFailed => DatabaseErrorCategory::Transaction,
        _ => DatabaseErrorCategory::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn representative_sqlx_errors_map_to_stable_categories() {
        assert_eq!(
            classify_sqlx_error(&sqlx::Error::PoolTimedOut),
            DatabaseErrorCategory::PoolTimeout
        );
        assert_eq!(
            classify_sqlx_error(&sqlx::Error::Protocol("private detail".to_owned())),
            DatabaseErrorCategory::Protocol
        );
        assert_eq!(
            classify_sqlx_error(&sqlx::Error::Decode("private detail".to_owned().into())),
            DatabaseErrorCategory::Decode
        );
        assert_eq!(
            classify_sqlx_error(&sqlx::Error::RowNotFound),
            DatabaseErrorCategory::Unknown
        );
    }

    #[test]
    fn category_display_never_contains_error_source_text() {
        let category = classify_sqlx_error(&sqlx::Error::Protocol("private detail".to_owned()));
        assert_eq!(category.to_string(), "protocol");
    }
}
