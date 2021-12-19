use crate::common::schema::fields::{FieldKind, ZetroField};

impl ZetroField {
    /// Gets the rust representation of a field's type.
    pub(super) fn to_rust_dtype(&self) -> String {
        let mut kind = match &self.kind {
            FieldKind::Int8 => String::from("i8"),
            FieldKind::UInt8 => String::from("u8"),
            FieldKind::Int16 => String::from("i16"),
            FieldKind::UInt16 => String::from("u16"),
            FieldKind::Int32 => String::from("i32"),
            FieldKind::UInt32 => String::from("u32"),
            FieldKind::Int64 => String::from("i64"),
            FieldKind::UInt64 => String::from("u64"),
            FieldKind::Float32 => String::from("f32"),
            FieldKind::Float64 => String::from("f64"),
            FieldKind::Boolean => String::from("bool"),
            FieldKind::StringValue => String::from("String"),
            FieldKind::StructValue(s) => {
                if self.is_recursive {
                    // Recursive types need to be boxed
                    format!("Box<{}>", s)
                } else {
                    s.to_owned()
                }
            }
            FieldKind::EnumValue(e) => e.to_owned(),
            FieldKind::NestedObject(s) => s.name.to_owned(),
        };

        if self.is_multiple {
            kind = format!("Vec<{}>", kind);
        }
        if self.is_nullable {
            kind = format!("Option<{}>", kind);
        }

        kind
    }
}
