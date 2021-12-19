use crate::common::schema::fields::{FieldKind, ZetroField};

impl ZetroField {
    /// Gets the typescript representation of a field's type.
    ///
    /// This function _does not_ represent type "nullability" because
    /// the syntax for that depends on context.
    /// In signature-like definitions (such as interfaces and function
    /// arguments), one must use a question token. eg. `fieldName?: fieldType`
    /// In all other contexts, one must use the `| null` suffix. eg.
    /// `fieldName | null`.
    pub(super) fn to_ts_dtype(&self) -> String {
        let mut kind = match &self.kind {
            FieldKind::Int8 => String::from("number"),
            FieldKind::UInt8 => String::from("number"),
            FieldKind::Int16 => String::from("number"),
            FieldKind::UInt16 => String::from("number"),
            FieldKind::Int32 => String::from("number"),
            FieldKind::UInt32 => String::from("number"),
            FieldKind::Int64 => String::from("number"),
            FieldKind::UInt64 => String::from("number"),
            FieldKind::Float32 => String::from("number"),
            FieldKind::Float64 => String::from("number"),
            FieldKind::Boolean => String::from("boolean"),
            FieldKind::StringValue => String::from("string"),
            // We treat enums as numbers because the built-in enum type
            // in TS is rather heavy.
            FieldKind::EnumValue(_) => String::from("number"),
            FieldKind::StructValue(s) => s.to_owned(),
            FieldKind::NestedObject(s) => s.name.to_owned(),
        };

        if self.is_multiple {
            kind = format!("{}[]", kind);
        }

        kind
    }
}
