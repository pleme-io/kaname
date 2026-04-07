//! Typed error variants for kaname operations.

/// Errors that can occur in kaname MCP server operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum KanameError {
    /// A requested configuration key was not found.
    #[error("config key not found: {key}")]
    ConfigKeyNotFound {
        /// The key that was looked up.
        key: String,
    },

    /// JSON serialization or deserialization failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// An I/O error occurred (e.g. during transport setup).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_key_not_found_display() {
        let err = KanameError::ConfigKeyNotFound {
            key: "appearance.theme".to_string(),
        };
        assert_eq!(err.to_string(), "config key not found: appearance.theme");
    }

    #[test]
    fn config_key_not_found_debug_contains_key() {
        let err = KanameError::ConfigKeyNotFound {
            key: "font_size".to_string(),
        };
        let debug = format!("{err:?}");
        assert!(debug.contains("font_size"));
        assert!(debug.contains("ConfigKeyNotFound"));
    }

    #[test]
    fn json_error_from_serde() {
        let bad: Result<serde_json::Value, _> = serde_json::from_str("not json {{{");
        let serde_err = bad.unwrap_err();
        let err = KanameError::from(serde_err);
        let display = err.to_string();
        assert!(display.starts_with("json error:"));
    }

    #[test]
    fn io_error_from_std() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = KanameError::from(io_err);
        let display = err.to_string();
        assert!(display.starts_with("io error:"));
        assert!(display.contains("file missing"));
    }

    #[test]
    fn config_key_not_found_with_empty_key() {
        let err = KanameError::ConfigKeyNotFound {
            key: String::new(),
        };
        assert_eq!(err.to_string(), "config key not found: ");
    }

    #[test]
    fn json_error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<KanameError>();
    }

    #[test]
    fn json_variant_converts_via_question_mark() {
        fn try_parse() -> Result<(), KanameError> {
            let _: serde_json::Value = serde_json::from_str("{}")?;
            Ok(())
        }
        assert!(try_parse().is_ok());
    }

    #[test]
    fn json_variant_converts_error_via_question_mark() {
        fn try_parse() -> Result<(), KanameError> {
            let _: serde_json::Value = serde_json::from_str("bad")?;
            Ok(())
        }
        assert!(try_parse().is_err());
    }

    #[test]
    fn io_variant_converts_via_question_mark() {
        fn try_io() -> Result<(), KanameError> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "boom"))?
        }
        assert!(try_io().is_err());
    }

    #[test]
    fn error_is_non_exhaustive() {
        fn _assert_match(e: KanameError) -> String {
            match e {
                KanameError::ConfigKeyNotFound { key } => key,
                KanameError::Json(e) => e.to_string(),
                KanameError::Io(e) => e.to_string(),
                _ => "unknown".to_string(),
            }
        }
    }
}
