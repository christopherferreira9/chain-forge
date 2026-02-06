//! Validation helpers for Chain Forge.

use std::fmt;

/// Error for invalid name format
#[derive(Debug, Clone)]
pub struct InvalidNameError {
    pub value: String,
    pub reason: String,
}

impl fmt::Display for InvalidNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid name '{}': {}. Use only lowercase letters, numbers, and hyphens (e.g., 'my-node-1').",
            self.value, self.reason
        )
    }
}

impl std::error::Error for InvalidNameError {}

/// Validates that a name follows the format: lowercase letters, numbers, and hyphens only.
/// Examples of valid names: "my-node", "dev-1", "test-server-2"
/// Examples of invalid names: "My Node", "test_node", "node@1"
pub fn validate_name(name: &str) -> Result<(), InvalidNameError> {
    if name.is_empty() {
        return Err(InvalidNameError {
            value: name.to_string(),
            reason: "name cannot be empty".to_string(),
        });
    }

    if name.starts_with('-') || name.ends_with('-') {
        return Err(InvalidNameError {
            value: name.to_string(),
            reason: "name cannot start or end with a hyphen".to_string(),
        });
    }

    if name.contains("--") {
        return Err(InvalidNameError {
            value: name.to_string(),
            reason: "name cannot contain consecutive hyphens".to_string(),
        });
    }

    for c in name.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '-' {
            let reason = if c.is_ascii_uppercase() {
                "uppercase letters are not allowed, use lowercase".to_string()
            } else if c == ' ' {
                "spaces are not allowed, use hyphens instead".to_string()
            } else if c == '_' {
                "underscores are not allowed, use hyphens instead".to_string()
            } else {
                format!("character '{}' is not allowed", c)
            };

            return Err(InvalidNameError {
                value: name.to_string(),
                reason,
            });
        }
    }

    Ok(())
}

/// Sanitizes a name by converting to lowercase, replacing spaces with hyphens,
/// and removing invalid characters.
pub fn sanitize_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c == ' ' || c == '_' {
                '-'
            } else if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                ' ' // Will be filtered out
            }
        })
        .filter(|c| *c != ' ')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_names() {
        assert!(validate_name("my-node").is_ok());
        assert!(validate_name("dev-1").is_ok());
        assert!(validate_name("test-server-2").is_ok());
        assert!(validate_name("node").is_ok());
        assert!(validate_name("a").is_ok());
        assert!(validate_name("123").is_ok());
        assert!(validate_name("my-node-123").is_ok());
    }

    #[test]
    fn test_invalid_names() {
        assert!(validate_name("").is_err());
        assert!(validate_name("My Node").is_err());
        assert!(validate_name("my_node").is_err());
        assert!(validate_name("my node").is_err());
        assert!(validate_name("-node").is_err());
        assert!(validate_name("node-").is_err());
        assert!(validate_name("my--node").is_err());
        assert!(validate_name("MyNode").is_err());
        assert!(validate_name("node@1").is_err());
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("My Node"), "my-node");
        assert_eq!(sanitize_name("my_node"), "my-node");
        assert_eq!(sanitize_name("My  Node"), "my-node");
        assert_eq!(sanitize_name("  my  node  "), "my-node");
        assert_eq!(sanitize_name("MyNode123"), "mynode123");
        assert_eq!(sanitize_name("node@#$%test"), "nodetest");
        assert_eq!(sanitize_name("--my--node--"), "my-node");
    }
}
