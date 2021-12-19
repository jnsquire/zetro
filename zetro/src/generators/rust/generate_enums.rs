use crate::common::schema::enums::ZetroEnum;

pub(super) fn generate_enums(scope: &mut Vec<String>, enums: &Vec<ZetroEnum>) {
    for _enum in enums {
        scope.push(generate_enum(&_enum));
    }
}

/// Generates a rust enum from the corresponding zetro enum and
/// adds it to scope
fn generate_enum(_enum: &ZetroEnum) -> String {
    let mut enum_variants: Vec<String> = Vec::new();

    for (i, variant_name) in _enum.variants.iter().enumerate() {
        enum_variants.push(format!("\t{} = {},", variant_name, i));
    }

    format!(
        "{}\n{}\npub enum {} {{\n{}\n}}",
        "#[derive(Debug, Copy, Clone, PartialEq, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]",
        "#[repr(u8)]",
        _enum.name,
        enum_variants.join("\n"),
    )
}
