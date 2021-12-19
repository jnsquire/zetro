use crate::common::schema::{fields::FieldKind, structs::ZetroStruct};

pub(super) fn generate_structs(
    scope: &mut Vec<String>,
    structs: &Vec<ZetroStruct>,
    untagged_repr: bool,
) {
    for _struct in structs {
        scope.extend(generate_struct(&_struct, untagged_repr));
        if untagged_repr {
            scope.extend(generate_untagged_serializer(&_struct));
            scope.extend(generate_untagged_deserializer(&_struct));
        }
    }
}

/// Generates a rust struct from the corresponding zetro struct and
/// adds it to scope
pub(super) fn generate_struct(_struct: &ZetroStruct, untagged_repr: bool) -> Vec<String> {
    let mut struct_blocks: Vec<String> = Vec::new();
    // List of fields for this struct
    let mut struct_fields: Vec<String> = Vec::new();

    for field in &_struct.fields {
        if let FieldKind::NestedObject(s) = &field.kind {
            // Another struct must be created for the nested object.
            generate_struct(s, untagged_repr)
                .into_iter()
                .enumerate()
                .for_each(|(i, mut _s)| {
                    // Add a #[allow(non_camel_case_types)] to the first
                    // struct because it contains an underscore
                    if i == 0 {
                        _s = format!("#[allow(non_camel_case_types)]\n{}", _s)
                    }
                    struct_blocks.push(_s);
                });
        }
        struct_fields.push(format!(
            "{}\tpub {}: {},",
            // Field documentation
            if let Some(d) = &field.description {
                format!("\t/// {}\n", d)
            } else {
                String::new()
            },
            field.name,
            field.to_rust_dtype()
        ));
    }

    struct_blocks.push(format!(
        "/// {}\n{}\npub struct {} {{\n{}\n}}",
        &_struct.description,
        if untagged_repr {
            "#[derive(Debug, Clone)]"
        } else {
            "#[derive(Debug, Clone, serde_derive::Serialize, serde_derive::Deserialize)]\n#[serde(rename_all = \"camelCase\")]"
        },
        _struct.name,
        struct_fields.join("\n"),
    ));

    struct_blocks
}

/// Generates an untagged serde serializer from the corresponding zetro struct
pub(super) fn generate_untagged_serializer(_struct: &ZetroStruct) -> Vec<String> {
    let mut impl_blocks: Vec<String> = Vec::new();
    let mut serialize_fn_elems: Vec<String> = Vec::new();

    // Serialize each item one by one
    for field in &_struct.fields {
        if let FieldKind::NestedObject(o) = &field.kind {
            // A serializer must be created for the nested object.
            impl_blocks.extend(generate_untagged_serializer(o));
        }
        serialize_fn_elems.push(format!(
            "\t\tstate.serialize_element(&self.{})?;",
            field.name
        ));
    }

    let serialize_fn = format!(
        "fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
\twhere S: serde::ser::Serializer,
\t{{
\t\tlet mut state = serializer.serialize_tuple(2)?;
{}
\t\tstate.end()    
\t}}",
        serialize_fn_elems.join("\n")
    );

    impl_blocks.push(format!(
        "impl {} for {} {{\n{}\n}}",
        "serde::ser::Serialize", _struct.name, serialize_fn
    ));
    impl_blocks
}

/// Generates an untagged serde deserializer from the corresponding zetro struct
pub(super) fn generate_untagged_deserializer(_struct: &ZetroStruct) -> Vec<String> {
    let mut impl_blocks: Vec<String> = Vec::new();
    let mut deserialize_parse_blocks: Vec<String> = Vec::new();

    // Add line-by-line parsing for function
    for field in &_struct.fields {
        if let FieldKind::NestedObject(o) = &field.kind {
            // A serializer must be created for the nested object.
            impl_blocks.extend(generate_untagged_deserializer(o))
        }

        deserialize_parse_blocks.push(format!(
            "\t\t\t\tlet {0} = seq.next_element::<{1}>()?;
\t\t\t\tif {0}.is_none() {{
\t\t\t\t\treturn Err(serde::de::Error::custom(\"invalid field\"));
\t\t\t\t}}
\t\t\t\tlet {0} = {0}.unwrap();\n",
            field.name,
            field.to_rust_dtype()
        ));
    }

    // Return value for deserializer fn
    let mut return_value_fields: Vec<String> = Vec::new();
    for field in &_struct.fields {
        return_value_fields.push(format!("\t\t\t\t\t{0}: {0},", &field.name));
    }

    let deserialize_fn = format!(
        "fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
\twhere D: serde::Deserializer<'de>,
\t{{
\t\tstruct Visitor;
\t\timpl<'de> serde::de::Visitor<'de> for Visitor {{
\t\t\ttype Value = {0};

\t\t\tfn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {{
\t\t\t\twrite!(formatter, \"\")
\t\t\t}}

\t\t\tfn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
\t\t\twhere A: serde::de::SeqAccess<'de>,
\t\t\t{{
{1}
\t\t\t\tOk({0} {{
{2}
\t\t\t\t}})
\t\t\t}}
\t\t}}
\t\tdeserializer.deserialize_tuple({3}, Visitor)
\t}}",
        _struct.name,
        deserialize_parse_blocks.join("\n"),
        return_value_fields.join("\n"),
        _struct.fields.len(),
        // serialize_fn_elems.join("\n")
    );

    impl_blocks.push(format!(
        "impl<'de> serde::de::Deserialize<'de> for {} {{\n{}\n}}",
        _struct.name, deserialize_fn
    ));
    impl_blocks
}
