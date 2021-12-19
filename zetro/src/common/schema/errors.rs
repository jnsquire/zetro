#[derive(Debug)]
pub(crate) struct SchemaError {
    pub kind: ErrorKind,
    pub offender: Offender,
}

#[derive(Debug)]
pub(crate) enum Offender {
    Field(String, String), // (struct/enum/route name, field name)
    Struct(String),        // (struct name)
    Enum(String),          // (enum name)
    Route(String),         // (route name)
    File(String),
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    InvalidReference(String),      // (name of invalid reference)
    UnrecognizedField(String),     // (field name)
    MissingField(String),          // (field name)
    BadFieldValue(String, String), // (field name, expected type)
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string_first_part = match &self.kind {
            ErrorKind::InvalidReference(ref_name) => format!("Invalid reference '{}'", ref_name),
            ErrorKind::UnrecognizedField(field_name) => {
                format!("Unrecognized field '{}'", field_name)
            }
            ErrorKind::MissingField(field_name) => {
                format!("Missing required field '{}'", field_name)
            }
            ErrorKind::BadFieldValue(field_name, expected_type) => format!(
                "Invalid value for field '{}'. Expected type: {}",
                field_name, expected_type
            ),
        };
        let string_second_part = match &self.offender {
            Offender::Field(parent_name, field_name) => {
                format!("Field {}.{}", parent_name, field_name)
            }
            Offender::Struct(struct_name) => format!("Struct '{}'", struct_name),
            Offender::Enum(enum_name) => format!("Enum '{}'", enum_name),
            Offender::Route(route_name) => format!("Route '{}'", route_name),
            Offender::File(file_name) => format!("File '{}'", file_name),
        };

        write!(
            f,
            "{}\nOffender was: {}",
            string_first_part, string_second_part
        )
    }
}
