use self::{
    enums::ZetroEnum,
    errors::{ErrorKind, Offender, SchemaError},
    fields::{FieldKind, ZetroField},
    routes::ZetroRoute,
    structs::ZetroStruct,
};

pub(crate) mod enums;
pub(crate) mod errors;
pub(crate) mod fields;
pub(crate) mod routes;
pub(crate) mod structs;

/// Represents the format of a schema JSON file.
#[derive(Debug, Clone)]
pub(crate) struct ZetroSchema {
    pub structs: Vec<ZetroStruct>,
    pub enums: Vec<ZetroEnum>,
    pub queries: Vec<ZetroRoute>,
    pub mutations: Vec<ZetroRoute>,
}

/// Stores all the structs in the current schema. Used to check for invalid
/// structs.
type ReferenceManifest<'a> = std::collections::HashMap<&'a String, bool>;

impl ZetroSchema {
    pub fn from_value(value: &serde_json::Value) -> Result<Self, SchemaError> {
        use serde_json::{Map, Value};

        let value = match value.as_object() {
            Some(v) => v,
            None => {
                return Err(SchemaError {
                    kind: ErrorKind::BadFieldValue(
                        String::from("schema.json"),
                        String::from("an object"),
                    ),
                    offender: Offender::File(String::from("schema.json")),
                });
            }
        };

        let mut schema_structs: Option<&Map<String, Value>> = None;
        let mut schema_enums: Option<&Map<String, Value>> = None;
        let mut schema_routes: Option<&Map<String, Value>> = None;

        for (key, value) in value {
            match key.as_str() {
                "structs" => match value.as_object() {
                    Some(v) => schema_structs = Some(v),
                    None => {
                        return Err(SchemaError {
                            kind: ErrorKind::BadFieldValue(
                                String::from("structs"),
                                String::from("an object"),
                            ),
                            offender: Offender::File(String::from("schema.json")),
                        });
                    }
                },
                "enums" => match value.as_object() {
                    Some(v) => schema_enums = Some(v),
                    None => {
                        return Err(SchemaError {
                            kind: ErrorKind::BadFieldValue(
                                String::from("enums"),
                                String::from("an object"),
                            ),
                            offender: Offender::File(String::from("schema.json")),
                        });
                    }
                },
                "routes" => match value.as_object() {
                    Some(v) => schema_routes = Some(v),
                    None => {
                        return Err(SchemaError {
                            kind: ErrorKind::BadFieldValue(
                                String::from("routes"),
                                String::from("an object"),
                            ),
                            offender: Offender::File(String::from("schema.json")),
                        });
                    }
                },
                _ => {
                    return Err(SchemaError {
                        kind: ErrorKind::UnrecognizedField(key.clone()),
                        offender: Offender::File(String::from("schema.json")),
                    });
                }
            }
        }

        let mut structs: Vec<ZetroStruct> = Vec::new();
        let mut enums: Vec<ZetroEnum> = Vec::new();
        let mut queries: Vec<ZetroRoute> = Vec::new();
        let mut mutations: Vec<ZetroRoute> = Vec::new();

        if let Some(schema_structs) = schema_structs {
            for (struct_name, fields) in schema_structs {
                let _struct = ZetroStruct::from_value(struct_name.to_owned(), fields)?;
                if _struct.is_nullable {
                    return Err(SchemaError {
                        kind: ErrorKind::BadFieldValue(
                            String::from("nullable"),
                            String::from("[empty]"),
                        ),
                        offender: Offender::Struct(struct_name.to_owned()),
                    });
                }
                if _struct.is_multiple {
                    return Err(SchemaError {
                        kind: ErrorKind::BadFieldValue(
                            String::from("multiple"),
                            String::from("[empty]"),
                        ),
                        offender: Offender::Struct(struct_name.to_owned()),
                    });
                }
                structs.push(_struct);
            }
        }
        if let Some(schema_enums) = schema_enums {
            for (enum_name, variants) in schema_enums {
                enums.push(ZetroEnum::from_value(enum_name.to_owned(), variants)?);
            }
        }
        if let Some(schema_routes) = schema_routes {
            for (route_name, fields) in schema_routes {
                let route = ZetroRoute::from_value(route_name.to_owned(), fields)?;
                match &route.kind {
                    routes::RouteKind::Query => queries.push(route),
                    routes::RouteKind::Mutation => mutations.push(route),
                }
            }
        }

        let schema = Self {
            enums,
            mutations,
            queries,
            structs,
        };

        schema.check_schema()?;

        Ok(schema)
    }

    /// Checks schema for invalid references
    fn check_schema(&self) -> Result<(), SchemaError> {
        let mut struct_manifest: ReferenceManifest = std::collections::HashMap::new();
        let mut enum_manifest: ReferenceManifest = std::collections::HashMap::new();

        for _enum in &self.enums {
            enum_manifest.insert(&_enum.name, true);
        }
        for _struct in &self.structs {
            struct_manifest.insert(&_struct.name, true);
        }
        for _struct in &self.structs {
            Self::check_struct(&struct_manifest, &enum_manifest, _struct)?
        }
        for route in &self.queries {
            Self::check_field(&struct_manifest, &enum_manifest, &route.request_body)?;
            Self::check_field(&struct_manifest, &enum_manifest, &route.response_body)?;
        }
        for route in &self.mutations {
            Self::check_field(&struct_manifest, &enum_manifest, &route.request_body)?;
            Self::check_field(&struct_manifest, &enum_manifest, &route.response_body)?;
        }

        Ok(())
    }

    fn check_struct(
        struct_manifest: &ReferenceManifest,
        enum_manifest: &ReferenceManifest,
        _struct: &ZetroStruct,
    ) -> Result<(), SchemaError> {
        Ok(for field in &_struct.fields {
            Self::check_field(struct_manifest, enum_manifest, field)?;
        })
    }

    fn check_field(
        struct_manifest: &ReferenceManifest,
        enum_manifest: &ReferenceManifest,
        field: &ZetroField,
    ) -> Result<(), SchemaError> {
        Ok(match &field.kind {
            FieldKind::StructValue(struct_name) => {
                if struct_manifest.get(struct_name).is_none() {
                    return Err(SchemaError {
                        kind: ErrorKind::InvalidReference(struct_name.to_owned()),
                        offender: Offender::Field(struct_name.to_owned(), field.name.to_owned()),
                    });
                }
            }
            FieldKind::EnumValue(enum_name) => {
                if enum_manifest.get(enum_name).is_none() {
                    return Err(SchemaError {
                        kind: ErrorKind::InvalidReference(enum_name.to_owned()),
                        offender: Offender::Field(enum_name.to_owned(), field.name.to_owned()),
                    });
                }
            }
            FieldKind::NestedObject(obj) => {
                Self::check_struct(struct_manifest, enum_manifest, obj)?
            }
            _ => {}
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{ErrorKind, ZetroSchema};

    /// Ensure invalid references are declined
    #[test]
    fn invalid_references() {
        // Structs
        let err = ZetroSchema::from_value(&json!({
            "structs": json!({
                "SomeStruct": json!({
                    "description": "A valid description",
                    "fields": json!({
                        "invalidRef": "struct~InvalidRef"
                    }),
                })
            }),
            "enums": json!({}),
            "routes": json!({}),
        }))
        .err()
        .expect("expected schema error(struct)");

        match &err.kind {
            &ErrorKind::InvalidReference(_) => {}
            _ => panic!(
                "expected schema error(struct) to be 'invalid reference'. Got: {:#?}",
                err
            ),
        }

        // Enums
        let err = ZetroSchema::from_value(&json!({
            "structs": json!({
                "SomeStruct": json!({
                    "description": "A valid description",
                    "fields": json!({
                        "invalidRef": "enum~InvalidRef"
                    }),
                })
            }),
            "enums": json!({}),
            "routes": json!({}),
        }))
        .err()
        .expect("expected schema error(enum)");

        match &err.kind {
            &ErrorKind::InvalidReference(_) => {}
            _ => panic!(
                "expected schema error(enum) to be 'invalid reference'. Got: {:#?}",
                err
            ),
        }

        // Nested structs
        let err = ZetroSchema::from_value(&json!({
            "structs": json!({
                "SomeStruct": json!({
                    "description": "A valid description",
                    "fields": json!({
                        "nestedObj": json!({
                            "description": "Another valid description",
                            "fields": json!({
                                "someField": "enum~InvalidRef",
                            }),
                        }),
                    }),
                }),
            }),
            "enums": json!({}),
            "routes": json!({}),
        }))
        .err()
        .expect("expected schema error(nested struct)");

        match &err.kind {
            &ErrorKind::InvalidReference(_) => {}
            _ => panic!(
                "expected schema error(nested struct) to be 'invalid reference'. Got: {:#?}",
                err
            ),
        }
    }
}
