use crate::common::schema::ErrorKind;

use super::{FieldKind, Offender, SchemaError, ZetroField};

/// Represents a single API route
#[derive(Debug, Clone)]
pub(crate) struct ZetroRoute {
    pub kind: RouteKind,
    pub name: String,
    pub description: String,
    pub request_body: ZetroField,
    pub response_body: ZetroField,
}

impl ZetroRoute {
    pub fn from_value(route_name: String, value: &serde_json::Value) -> Result<Self, SchemaError> {
        match value.as_object() {
            Some(v) => {
                let kind = match v.get("kind") {
                    Some(v) => RouteKind::from_value(route_name.clone(), v)?,
                    None => {
                        return Err(SchemaError {
                            kind: ErrorKind::MissingField(String::from("kind")),
                            offender: Offender::Route(route_name),
                        })
                    }
                };
                let description = match v.get("description") {
                    Some(v) => match v.as_str() {
                        Some(v) => v,
                        None => {
                            return Err(SchemaError {
                                kind: ErrorKind::BadFieldValue(
                                    String::from("description"),
                                    String::from("string"),
                                ),
                                offender: Offender::Field(route_name, String::from("description")),
                            });
                        }
                    },
                    None => {
                        return Err(SchemaError {
                            kind: ErrorKind::MissingField(String::from("description")),
                            offender: Offender::Route(route_name),
                        });
                    }
                };

                let request_body = match v.get("request") {
                    Some(v) => {
                        ZetroField::from_value(route_name.clone(), String::from("request"), v)?
                    }
                    None => {
                        return Err(SchemaError {
                            kind: ErrorKind::MissingField(String::from("request")),
                            offender: Offender::Route(route_name),
                        });
                    }
                };
                if let FieldKind::NestedObject(_) = request_body.kind {
                    return Err(SchemaError {
                        kind: ErrorKind::BadFieldValue(
                            String::from("request"),
                            String::from("not a nested object"),
                        ),
                        offender: Offender::Route(route_name),
                    });
                }

                let response_body = match v.get("response") {
                    Some(v) => {
                        ZetroField::from_value(route_name.clone(), String::from("response"), v)?
                    }
                    None => {
                        return Err(SchemaError {
                            kind: ErrorKind::MissingField(String::from("response")),
                            offender: Offender::Route(route_name),
                        });
                    }
                };
                if let FieldKind::NestedObject(_) = response_body.kind {
                    return Err(SchemaError {
                        kind: ErrorKind::BadFieldValue(
                            String::from("response"),
                            String::from("not a nested object"),
                        ),
                        offender: Offender::Route(route_name),
                    });
                }

                Ok(Self {
                    name: route_name,
                    description: description.to_string(),
                    kind,
                    request_body,
                    response_body,
                })
            }
            None => {
                return Err(SchemaError {
                    kind: ErrorKind::BadFieldValue(route_name.clone(), String::from("an object")),
                    offender: Offender::Route(route_name),
                });
            }
        }
    }

    /// Returns the encrypted and base64-encoded version of a route.
    /// We encrypt the route name to make reverse engineering more difficult.
    pub(crate) fn encrypt_route_name(&self) -> String {
        let mut route_encrypted =
            crypto::hmac::Hmac::new(crypto::sha1::Sha1::new(), "zetro".as_bytes());
        crypto::mac::Mac::input(&mut route_encrypted, self.name.as_bytes());

        let route_encrypted = base64::encode_config(
            crypto::mac::Mac::result(&mut route_encrypted).code(),
            base64::URL_SAFE_NO_PAD,
        );

        route_encrypted
    }
}

#[derive(Debug, Clone)]
pub(crate) enum RouteKind {
    /// A query means no content is changed in the api call.
    /// eg. fetching a list of videos
    Query,
    /// A mutation means content is created, updated, or deleted
    /// eg. liking a video
    Mutation,
}

impl RouteKind {
    pub fn to_method_code(&self) -> u8 {
        match self {
            RouteKind::Query => 1,
            RouteKind::Mutation => 2,
        }
    }
}

impl RouteKind {
    pub fn from_value(route_name: String, value: &serde_json::Value) -> Result<Self, SchemaError> {
        match value.as_str() {
            Some(v) => match v {
                "query" => Ok(Self::Query),
                "mutation" => Ok(Self::Mutation),
                _ => Err(SchemaError {
                    kind: ErrorKind::BadFieldValue(
                        String::from("kind"),
                        String::from("one of 'query' or 'mutation'"),
                    ),
                    offender: Offender::Field(route_name, String::from("kind")),
                }),
            },
            None => Err(SchemaError {
                kind: ErrorKind::BadFieldValue(String::from("kind"), String::from("a string")),
                offender: Offender::Field(route_name, String::from("kind")),
            }),
        }
    }
}
