use super::{ErrorKind, Offender, SchemaError};

/// Denotes an enum.
#[derive(Debug, Clone)]
pub(crate) struct ZetroEnum {
    pub name: String,
    pub variants: Vec<String>,
}

impl ZetroEnum {
    pub fn from_value(enum_name: String, value: &serde_json::Value) -> Result<Self, SchemaError> {
        if let Some(array) = value.as_array() {
            let mut variants: Vec<String> = Vec::new();

            for variant in array {
                if let Some(s) = variant.as_str() {
                    variants.push(s.to_string());
                } else {
                    return Err(SchemaError {
                        kind: ErrorKind::BadFieldValue(
                            enum_name.clone(),
                            String::from("a list of strings"),
                        ),
                        offender: Offender::Enum(enum_name),
                    });
                }
            }

            Ok(Self {
                name: enum_name,
                variants,
            })
        } else {
            return Err(SchemaError {
                kind: ErrorKind::BadFieldValue(enum_name.clone(), String::from("list of strings")),
                offender: Offender::Enum(enum_name),
            });
        }
    }
}
