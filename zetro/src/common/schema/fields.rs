use super::{
    structs::{self, ZetroStruct},
    ErrorKind, Offender, SchemaError,
};

#[derive(Debug, Clone)]
pub(crate) enum FieldKind {
    Int8,
    UInt8,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Float32,
    Float64,
    Boolean,
    StringValue,
    StructValue(String),       // Struct name
    EnumValue(String),         // Snum name
    NestedObject(ZetroStruct), // A nested object
}

/// Denotes a single field in a schema. Fields will always be of string type.
/// The syntax for a field is as follows:
///     <nullable><multiple><dtype>~<extra>; <description>
/// To represent a string, we would write:
///     string
/// To represent a list of enums we would write:
///     []enum~EnumNameHere
/// To represent a nullable list of structs with a description we would write:
///     ?[]struct~StructNameHere; Description here.
///
/// The nullable operator (`?`) must always be the first character (if present).
/// followed by the multiple operator (`[]`), if present.
/// The description, if present must be put after a semicolon-space construct,
/// like this:
///     ?string; A nullable string. Notice the semicolon and space
#[derive(Debug, Clone)]
pub(crate) struct ZetroField {
    /// Optional description for the field
    pub description: Option<String>,
    /// Property name for field
    pub name: String,
    /// Kind of field and extra information for non-primitive fields
    /// ie. structs, enums and nested objects
    pub kind: FieldKind,
    /// Can this field be null?
    pub is_nullable: bool,
    /// Is this a list of items?
    pub is_multiple: bool,
    /// Whether the field is recursive. ie. It references its _direct_ parent
    pub is_recursive: bool,
}

impl ZetroField {
    pub fn from_value(
        struct_name: String,
        field_name: String,
        value: &serde_json::Value,
    ) -> Result<Self, SchemaError> {
        if value.is_object() {
            // A new struct has to be generated for nested values
            let nested_struct_name =
                structs::generate_nested_struct_name(&struct_name, &field_name);
            let nested_struct = ZetroStruct::from_value(nested_struct_name, value)?;

            return Ok(Self {
                is_nullable: nested_struct.is_nullable.clone(),
                is_recursive: false,
                is_multiple: nested_struct.is_multiple.clone(),
                kind: FieldKind::NestedObject(nested_struct),
                description: None,
                name: field_name,
            });
        }

        let mut value = match value.as_str() {
            Some(v) => v,
            None => todo!(),
        };

        let kind: FieldKind;
        let is_nullable: bool;
        let is_multiple: bool;
        let mut is_recursive = false;
        let description: Option<String>;

        if value.starts_with("?") {
            is_nullable = true;
            value = &value[1..];
        } else {
            is_nullable = false;
        }
        if value.starts_with("[]") {
            is_multiple = true;
            value = &value[2..];
        } else {
            is_multiple = false;
        }

        // A value can be followed by a '; ' to denote description
        let _parts = value.split("; ").collect::<Vec<_>>();
        let _dtype_parts = _parts[0];
        if let Some(d) = _parts.get(1) {
            description = Some(d.to_string());
        } else {
            description = None;
        }

        // A value have a '~' to add extra information
        let _dtype_parts = _dtype_parts.split("~").collect::<Vec<_>>();
        let dtype = *_dtype_parts.get(0).unwrap();
        let extra = _dtype_parts.get(1);

        if dtype == "string" {
            kind = FieldKind::StringValue;
        } else if dtype == "i8" {
            kind = FieldKind::Int8;
        } else if dtype == "u8" {
            kind = FieldKind::UInt8;
        } else if dtype == "i16" {
            kind = FieldKind::Int16;
        } else if dtype == "u16" {
            kind = FieldKind::UInt16;
        } else if dtype == "i32" {
            kind = FieldKind::Int32;
        } else if dtype == "u32" {
            kind = FieldKind::UInt32;
        } else if dtype == "i64" {
            kind = FieldKind::Int64;
        } else if dtype == "u64" {
            kind = FieldKind::UInt64;
        } else if dtype == "f32" {
            kind = FieldKind::Float32;
        } else if dtype == "f64" {
            kind = FieldKind::Float64;
        } else if dtype == "bool" {
            kind = FieldKind::Boolean;
        } else if dtype == "enum" {
            let enum_name = extra.expect("enum declarations must contain enum name");
            kind = FieldKind::EnumValue(enum_name.to_string());
        } else if dtype == "struct" {
            let _struct_name = extra.expect("struct declarations must contain struct name");
            is_recursive = _struct_name.to_string() == struct_name;
            if is_recursive && !is_nullable && !is_multiple {
                // Disallow recursive types to be both non-null and non-multiple
                // to avoid infinite recursion
                return Err(SchemaError {
                    kind: ErrorKind::BadFieldValue(
                        field_name.clone(),
                        String::from("nullable and/or multiple to avoid an infinite loop."),
                    ),
                    offender: Offender::Field(struct_name, field_name),
                });
            }
            kind = FieldKind::StructValue(_struct_name.to_string());
        } else {
            return Err(SchemaError {
                kind: ErrorKind::BadFieldValue(
                    field_name.clone(),
                    String::from("string or object"),
                ),
                offender: Offender::Field(struct_name, field_name),
            });
        }

        Ok(Self {
            description,
            kind,
            name: field_name,
            is_multiple,
            is_nullable,
            is_recursive,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::panic;

    use serde_json::json;

    use super::super::{FieldKind, ZetroField};

    /// Ensure field parsing works correctly
    #[test]
    fn check_field_parsing() {
        let struct_name = String::from("ExampleStruct");
        let field_name = String::from("exampleField");

        // 1. Parse a non-null string field
        let string_non_null = ZetroField::from_value(
            struct_name.clone(),
            field_name.clone(),
            &json!("string; a non-null string"),
        )
        .unwrap();

        match string_non_null.kind {
            FieldKind::StringValue => {}
            _ => {
                panic!("expected type to be string");
            }
        }
        assert_eq!(string_non_null.is_nullable, false);
        assert_eq!(string_non_null.is_multiple, false);
        assert_eq!(
            string_non_null.description,
            Some(String::from("a non-null string"))
        );

        // 2. Parse nullable list of enums
        let enum_list_nullable = ZetroField::from_value(
            struct_name.clone(),
            field_name.clone(),
            &json!("?[]enum~EnumName; a nullable enum"),
        )
        .unwrap();

        match enum_list_nullable.kind {
            FieldKind::EnumValue(enum_name) => {
                assert_eq!(enum_name, String::from("EnumName"));
            }
            _ => {
                panic!("expected type to be enum");
            }
        }
        assert_eq!(enum_list_nullable.is_nullable, true);
        assert_eq!(enum_list_nullable.is_multiple, true);
        assert_eq!(
            enum_list_nullable.description,
            Some(String::from("a nullable enum"))
        );

        // 3. Parse nested struct
        let nested_struct = ZetroField::from_value(
            struct_name.clone(),
            field_name.clone(),
            &json!({
                "description": "nested struct",
                "fields": {
                    "first": "i8",
                }
            }),
        )
        .unwrap();

        match nested_struct.kind {
            FieldKind::NestedObject(zetro_struct) => {
                match zetro_struct.fields[0].kind {
                    FieldKind::Int8 => {}
                    _ => {
                        panic!("expected type to be Int8");
                    }
                }
                assert_eq!(zetro_struct.fields[0].description, None);
                assert_eq!(zetro_struct.fields[0].is_multiple, false);
                assert_eq!(zetro_struct.fields[0].is_nullable, false);
                assert_eq!(zetro_struct.is_nullable, false);
                assert_eq!(zetro_struct.description, String::from("nested struct"));
            }
            _ => {
                panic!("expected type to be enum");
            }
        }
        assert_eq!(nested_struct.is_nullable, false);
        assert_eq!(nested_struct.is_multiple, false);
    }
}
