use crate::common::schema::ErrorKind;

use super::{Offender, SchemaError, ZetroField};

/// Represents a collection of items. Akin to an object in javascript
/// or a class in python.
#[derive(Debug, Clone)]
pub(crate) struct ZetroStruct {
    /// Name of the struct
    pub name: String,
    /// Mandatory description for this struct
    pub description: String,
    /// Is this struct nullable? Always false for top-level structs
    pub is_nullable: bool,
    /// Is this struct represented in a list? Always false for top-level structs
    pub is_multiple: bool,
    /// A list of fields in this struct. Always ordered alphabetically from
    /// uppercase A to lowercase z
    pub fields: Vec<ZetroField>,
}

impl ZetroStruct {
    pub fn from_value(struct_name: String, value: &serde_json::Value) -> Result<Self, SchemaError> {
        // Coerce value into map
        let value = match value.as_object() {
            Some(v) => v,
            None => {
                return Err(SchemaError {
                    kind: ErrorKind::BadFieldValue(struct_name.clone(), String::from("an object")),
                    offender: Offender::Struct(struct_name),
                });
            }
        };

        let mut description: Option<String> = None;
        let mut multiple: bool = false;
        let mut nullable: bool = false;
        let mut schema_fields: Option<&serde_json::Map<String, serde_json::Value>> = None;

        for (key, val) in value {
            match key.as_str() {
                "description" => match val.as_str() {
                    Some(v) => description = Some(v.to_owned()),
                    None => {
                        return Err(SchemaError {
                            kind: ErrorKind::BadFieldValue(
                                "description".to_string(),
                                String::from("a string"),
                            ),
                            offender: Offender::Field(struct_name, String::from("description")),
                        });
                    }
                },
                "multiple" => match val.as_bool() {
                    Some(v) => multiple = v,
                    None => {
                        return Err(SchemaError {
                            kind: ErrorKind::BadFieldValue(
                                "multiple".to_string(),
                                String::from("a boolean"),
                            ),
                            offender: Offender::Field(struct_name, "multiple".to_string()),
                        });
                    }
                },
                "nullable" => match val.as_bool() {
                    Some(v) => nullable = v,
                    None => {
                        return Err(SchemaError {
                            kind: ErrorKind::BadFieldValue(
                                "nullable".to_string(),
                                String::from("a boolean"),
                            ),
                            offender: Offender::Field(struct_name, "nullable".to_string()),
                        });
                    }
                },
                "fields" => match val.as_object() {
                    Some(v) => schema_fields = Some(v),
                    None => {
                        return Err(SchemaError {
                            kind: ErrorKind::BadFieldValue(
                                "fields".to_string(),
                                String::from("an object"),
                            ),
                            offender: Offender::Field(struct_name, "fields".to_string()),
                        });
                    }
                },
                _ => {
                    return Err(SchemaError {
                        kind: ErrorKind::UnrecognizedField(key.to_owned()),
                        offender: Offender::Struct(struct_name),
                    });
                }
            }
        }

        if description.is_none() {
            return Err(SchemaError {
                kind: ErrorKind::MissingField("description".to_string()),
                offender: Offender::Struct(struct_name),
            });
        }
        if schema_fields.is_none() {
            return Err(SchemaError {
                kind: ErrorKind::MissingField("fields".to_string()),
                offender: Offender::Field(struct_name, "fields".to_string()),
            });
        }
        let description = description.unwrap();
        let schema_fields = schema_fields.unwrap();

        let mut fields_sorted = schema_fields.keys().collect::<Vec<_>>();
        fields_sorted.sort();

        let mut fields: Vec<ZetroField> = Vec::new();

        for field_name in fields_sorted {
            let field_value = schema_fields.get(field_name).unwrap();

            fields.push(ZetroField::from_value(
                struct_name.clone(),
                field_name.to_owned(),
                field_value,
            )?);
        }

        Ok(Self {
            name: struct_name.to_owned(),
            is_multiple: multiple,
            is_nullable: nullable,
            description,
            fields,
        })
    }
}

pub(crate) fn generate_nested_struct_name(struct_name: &str, field_name: &str) -> String {
    format!("{}_{}", struct_name, field_name)
}

#[cfg(test)]
mod tests {
    use std::panic;

    use serde_json::json;

    use super::super::FieldKind;
    use crate::common::schema::structs::ZetroStruct;

    /// Fields must be sorted alphabetically to ensure deterministic array
    /// generation
    #[test]
    fn order_alphabetically() {
        let struct_json = json!({
            "description": "",
            "fields": {
                "bbb": "[]u32",
                "aac": "?struct~StructName",
                "zzz": "?[]enum~EnumName",
                "abc": {
                    "description": "nested abc struct",
                    "fields": {
                        "abc": "?i64",
                        "aaa": "string"
                    }
                },
                "aaa": "?string",
                "AAA": "?string",
            }
        });

        let obj = ZetroStruct::from_value(String::from("MyStruct"), &struct_json).unwrap();

        assert_eq!(obj.fields[0].name, String::from("AAA"));
        assert_eq!(obj.fields[1].name, String::from("aaa"));
        assert_eq!(obj.fields[2].name, String::from("aac"));
        assert_eq!(obj.fields[3].name, String::from("abc"));
        match obj.fields[3].clone().kind {
            FieldKind::NestedObject(obj) => {
                assert_eq!(obj.fields[0].name, String::from("aaa"));
                assert_eq!(obj.fields[1].name, String::from("abc"));
            }
            _ => {
                panic!("expected field to be a nested object");
            }
        }
        assert_eq!(obj.fields[4].name, String::from("bbb"));
        assert_eq!(obj.fields[5].name, String::from("zzz"));
    }

    /// Description is a mandatory field in structs. Reject structs with no
    /// description
    #[test]
    fn reject_no_description() {
        let json = json!({
            "fields": {
                "example": "string; example field"
            }
        });
        assert_eq!(
            ZetroStruct::from_value("TestStruct".to_string(), &json).is_err(),
            true
        );
    }

    /// Description is a mandatory field in structs. Reject nested structs with
    /// no description
    #[test]
    fn reject_nested_no_description() {
        let json = json!({
            "fields": {
                "example": "string; example field",
                "nested": {
                    "example2": "string; another example field",
                }
            }
        });
        assert_eq!(
            ZetroStruct::from_value("TestStruct".to_string(), &json).is_err(),
            true
        );
    }
}
