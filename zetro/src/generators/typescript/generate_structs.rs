use crate::common::schema::{fields::FieldKind, structs::ZetroStruct};

/// Generates typescript interfaces and [de]serialization functions for each
/// struct.
pub(super) fn generate_structs(
    scope: &mut Vec<String>,
    structs: &Vec<ZetroStruct>,
    untagged_repr: bool,
) {
    scope.push(String::from("/* ============ Structs ============ */"));
    for _struct in structs {
        scope.extend(generate_interface(&_struct, true));
        if untagged_repr {
            scope.extend(generate_untagged_serializer(&_struct, true));
            scope.extend(generate_untagged_deserializer(&_struct, true));
        }
    }
    scope.push(String::from("/* ============ End Structs ============ */"));
}

/// Generates typescript interfaces from given ZetroStruct. Nested objects get
/// their own interfaces.
pub(super) fn generate_interface(_struct: &ZetroStruct, exported: bool) -> Vec<String> {
    // List of interfaces
    let mut interfaces: Vec<String> = Vec::new();
    // List of interface fields for _this_ interface.
    let mut interface_fields: Vec<String> = Vec::new();

    for field in &_struct.fields {
        if let FieldKind::NestedObject(s) = &field.kind {
            // Generate interface for nested object
            interfaces.extend(generate_interface(s, false));
        }
        if let Some(description) = &field.description {
            interface_fields.push(format!("\t/** {} */", description));
        }
        interface_fields.push(format!(
            "\t{}{}: {},",
            field.name,
            if field.is_nullable { "?" } else { "" },
            field.to_ts_dtype(),
        ));
    }

    let interface = format!(
        "/** {} */\n{}interface {} {{\n{}\n}}",
        _struct.description,
        if exported { "export " } else { "" },
        _struct.name,
        interface_fields.join("\n"),
    );

    interfaces.push(interface);

    interfaces
}

/// Generates struct serializer (interface to list representation) from given
/// ZetroStruct. Nested objects get their own serializer functions.
pub(super) fn generate_untagged_serializer(_struct: &ZetroStruct, exported: bool) -> Vec<String> {
    // List of serialization functions
    let mut serializer_fns: Vec<String> = Vec::new();

    // List of elements in returned array.
    let mut ret_array_elems: Vec<String> = Vec::new();

    for field in &_struct.fields {
        // I could remove duplication in the match, but I think that makes it
        // more difficult to reason about the code.
        match &field.kind {
            FieldKind::StructValue(struct_name) => {
                // Call the serializer function of that struct.
                // Note that we don't create a serializer because it will be
                // created anyway.
                if field.is_multiple {
                    // Use a .map(). And use a regular function because it is
                    // faster than an arrow function.
                    ret_array_elems.push(format!(
                        "\t\tobj.{}{}.map(function (nested) {{ return serialize{}(nested); }})",
                        field.name,
                        if field.is_nullable { "?" } else { "" },
                        struct_name,
                    ));
                } else {
                    // Access property directly
                    ret_array_elems
                        .push(format!("\t\tserialize{}(obj.{})", struct_name, field.name));
                }
            }
            FieldKind::NestedObject(s) => {
                // Generate a serializer function for nested object

                serializer_fns.extend(generate_untagged_serializer(s, false));

                if field.is_multiple {
                    // Use a .map(). And use a regular function because it is
                    // faster than an arrow function.
                    ret_array_elems.push(format!(
                        "\t\tobj.{}{}.map(function (elem: any) {{ return serialize{}(elem); }})",
                        field.name,
                        if field.is_nullable { "?" } else { "" },
                        s.name,
                    ));
                } else {
                    // Access property directly
                    ret_array_elems.push(format!("\t\tserialize{}(obj.{})", s.name, field.name));
                }
            }
            _ => {
                // Primitives, even if they are multiple or nullable, can
                // be accessed directly.
                ret_array_elems.push(format!("\t\tobj.{}", field.name));
            }
        }
    }

    serializer_fns.push(format!(
        "{0}function serialize{1}(obj{2}: {1}): any[] | null {{{3}
\treturn [\n{4}\n\t];
}}",
        if exported { "export " } else { "" }, // Export the function?
        _struct.name,
        // Make argument optional for nullable structs
        if _struct.is_nullable { "?" } else { "" },
        if _struct.is_nullable {
            // Add a null check for nullable structs
            "\nif (obj == null) { return null; }"
        } else {
            ""
        },
        ret_array_elems.join(",\n"),
    ));

    serializer_fns
}

/// Generates struct deserializer (list representation to interface) from given
/// ZetroStruct. Nested objects get their own deserializer functions.
pub(super) fn generate_untagged_deserializer(_struct: &ZetroStruct, exported: bool) -> Vec<String> {
    // List of deserialization functions
    let mut deserializer_fns: Vec<String> = Vec::new();

    // List of properties in returned object.
    let mut ret_object_props: Vec<String> = Vec::new();

    // I could remove duplication in the match, but I think that makes it
    // more difficult to reason about the code.
    for (i, field) in _struct.fields.iter().enumerate() {
        match &field.kind {
            FieldKind::StructValue(struct_name) => {
                // Call the deserializer function of that struct.
                if field.is_multiple {
                    // Use a .map()
                    ret_object_props.push(format!(
                        "\t\t{}: obj[{}]{}.map(function (elem: any) {{ return deserialize{}(elem); }})",
                        field.name,
                        i, // Index of field, since we're accessing an array
                        if field.is_nullable { "?" } else { "" },
                        struct_name,
                    ));
                } else {
                    // Deserialize field without calling map
                    ret_object_props.push(format!(
                        "\t\t{}: deserialize{}(obj[{}])",
                        field.name, struct_name, i
                    ));
                }
            }
            FieldKind::NestedObject(s) => {
                // Generate a serializer function for nested object
                deserializer_fns.extend(generate_untagged_deserializer(s, false));

                if field.is_multiple {
                    // Use a .map()
                    ret_object_props.push(format!(
                        "\t\t{}: obj[{}]{}.map(function (elem: any) {{ return deserialize{}(elem); }})",
                        field.name,
                        i, // Index of field, since we're accessing an array
                        if field.is_nullable { "?" } else { "" },
                        s.name,
                    ));
                } else {
                    // Deserialize field without calling map
                    ret_object_props.push(format!(
                        "\t\t{}: deserialize{}(obj[{}])",
                        field.name, s.name, i
                    ));
                }
            }
            _ => {
                // Primitives, even if they are multiple or nullable, can
                // be accessed directly.
                ret_object_props.push(format!("\t\t{}: obj[{}]", field.name, i));
            }
        }
    }

    deserializer_fns.push(format!(
        "{0}function deserialize{1}(obj: any): {1} | null {{
\tif (obj == null) {{ return null; }}
\treturn {{\n{2}\n\t}};
}}",
        if exported { "export " } else { "" }, // Export the function?
        _struct.name,
        // Make argument optional for nullable structs
        ret_object_props.join(",\n"),
    ));

    deserializer_fns
}
