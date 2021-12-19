use crate::common::schema::enums::ZetroEnum;

/// Generates typescript enums from ZetroEnums.
///
/// Note that the generated "enum" is simply a read-only javascript object.
/// This is because **typescript compiles enums into javascript functions**,
/// which poses minification as well as performance problems.
pub(super) fn generate_enums(scope: &mut Vec<String>, enums: &Vec<ZetroEnum>) {
    scope.push(String::from("/* ============ Enums ============ */"));
    for _enum in enums {
        scope.push(generate_enum(&_enum));
    }
    scope.push(String::from("/* ============ End Enums ============ */"));
}

pub(super) fn generate_enum(_enum: &ZetroEnum) -> String {
    // A list of enum variants in property signature format
    let mut enum_variants: Vec<String> = Vec::new();

    for (i, variant) in _enum.variants.iter().enumerate() {
        enum_variants.push(format!("\t{}: {}", variant, i + 1));
    }

    let enum_block = format!(
        "export const {} = {{\n{}\n}} as const;",
        _enum.name,
        enum_variants.join(",\n")
    );

    enum_block
}
