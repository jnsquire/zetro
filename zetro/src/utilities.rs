use convert_case::Case;

pub(super) struct ZetroArgs {
    pub schema_file: String, // Path to schema file
    pub out_file: String,    // Path to output file
    pub language: EmitLang,
    pub field_casing: Option<Case>,
    pub plugins: Vec<PluginCall>,
    pub mangle: Option<bool>,
    pub untagged: bool,
}

pub(super) struct PluginCall {
    pub name: String,
    pub args: std::collections::HashMap<String, String>,
}

pub(super) enum EmitLang {
    Rust,
    Typescript,
}

impl EmitLang {
    /// Get language from file extension
    pub fn from_ext(ext: &str) -> Option<Self> {
        match ext {
            "ts" => Some(Self::Typescript),
            "tsx" => Some(Self::Typescript),
            "rs" => Some(Self::Rust),
            _ => None,
        }
    }
}

pub(crate) fn parse_bool(val: &str) -> Option<bool> {
    if val == "true" || val == "1" || val == "on" || val == "yes" || val == "t" {
        return Some(true);
    } else if val == "false" || val == "0" || val == "off" || val == "no" || val == "f" {
        return Some(false);
    }
    None
}

pub(super) fn parse_args(args: Vec<String>) -> Result<ZetroArgs, String> {
    let plugin_call_args =
        regex::Regex::new(r"(?i)([a-z0-9\-]+)(\(([a-z0-9\-]+:[a-z0-9\-]+ ?)+\))?").unwrap();

    let mut schema_file: Option<String> = None; // Path to schema file
    let mut out_file: Option<String> = None; // Path to output file
    let mut language: Option<EmitLang> = None; // Language
    let mut field_casing: Option<Case> = None; // Field casing
    let mut plugins: Vec<PluginCall> = Vec::new(); // Plugin list
    let mut should_mangle: Option<bool> = None; // Whether to mangle field names
    let mut untagged_repr = false; // Whether to generate "untagged", ie. array-based structs

    // Files must be of the form {file}_generated.{ext} or
    // {file}-generated.{ext} to avoid overwriting actual code.
    // This variable allows `out_file` to have any name
    let mut ignore_naming_convention = false;

    for arg in &args[1..] {
        let keyval = arg.split("=").collect::<Vec<_>>();
        let key = keyval[0];

        match key {
            "--schema" => {
                schema_file = Some(
                    match keyval.get(1) {
                        Some(&v) => v,
                        None => return Err(String::from("Expected path to schema file")),
                    }
                    .to_owned(),
                );
            }
            "--out-file" => {
                out_file = Some(
                    match keyval.get(1) {
                        Some(&v) => v,
                        None => return Err(String::from("Expected path to output file")),
                    }
                    .to_owned(),
                );
            }
            "--field-casing" => {
                field_casing = match keyval.get(1) {
                    Some(&v) => match v.to_lowercase().as_str() {
                        "snake" => Some(Case::Snake),
                        "camel" => Some(Case::Camel),
                        _ => {
                            return Err(format!(
                                "Expected field casing to be one of: {}\nGot: '{}'",
                                "'snake', 'camel'", v
                            ))
                        }
                    },
                    None => return Err(String::from("Expected arguments for field casing.")),
                };
            }
            "--mangle" => {
                should_mangle = match keyval.get(1) {
                    Some(&v) => {
                        let v = parse_bool(&v.to_lowercase());
                        if v.is_some() {
                            v
                        } else {
                            return Err(String::from("Invalid value for --mangle. Expected one of 'true', '1', 'false', '0'"));
                        }
                    }
                    None => return Err(String::from("Expected boolean value for --mangle")),
                };
            }
            "--untagged" => {
                untagged_repr = match keyval.get(1) {
                    Some(&v) => {
                        let v = parse_bool(&v.to_lowercase());
                        if let Some(v) = v {
                            v
                        } else {
                            return Err(String::from("Invalid value for --untagged. Expected one of 'true', '1', 'false', '0'"));
                        }
                    }
                    None => return Err(String::from("Expected boolean value for --untagged")),
                };
            }
            "--add-plugin" => {
                // Adds arguments to plugin: eg.
                // --add-plugin=example_plug(is_real:false,best_number:7);
                // Notice that spaces are not allowed and colons separate
                // keys and values.
                // Plugins can also be empty, like this:
                // --add-plugin=empty_plugin() or --add-plugin=empty_plugin
                let plugin_call = match keyval.get(1) {
                    Some(&v) => v,
                    None => {
                        return Err(String::from("--plugin must be followed by a plugin call."))
                    }
                };
                let plugin_call_reg = match plugin_call_args.captures_iter(plugin_call).next() {
                    Some(v) => v,
                    None => {
                        return Err(String::from("Invalid plugin call. Expected format: plugin-name(plugin-arg-1:true plugin-arg-2:42)"));
                    }
                };
                let plugin_name = match plugin_call_reg.get(1) {
                    Some(v) => v.as_str(),
                    None => {
                        return Err(String::from("Invalid plugin call: Missing plugin name"));
                    }
                }
                .to_lowercase();

                // Check if plugin already exists
                if plugins
                    .iter()
                    .find(|&elem| elem.name == plugin_name)
                    .is_some()
                {
                    return Err(format!("Duplicate plugin entry: {}", plugin_name));
                }

                let mut plugin_entry = PluginCall {
                    name: plugin_name,
                    args: std::collections::HashMap::new(),
                };

                // Parse arguments (if any)
                if let Some(v) = plugin_call_reg.get(2) {
                    let plugin_args = v.as_str();

                    // Strip starting and ending '(', ')' from args
                    let plugin_args = &plugin_args[1..plugin_args.len() - 1];

                    // Split by space to get individual arguments
                    for arg in plugin_args.split(" ") {
                        let arg = arg.split(":").collect::<Vec<_>>();
                        let key = match arg.get(0) {
                            Some(&v) => v,
                            None => {
                                return Err(String::from(
                                    "Invalid plugin call: Missing argument key",
                                ));
                            }
                        }
                        .to_lowercase();

                        // Check if key already exists
                        if plugin_entry.args.contains_key(&key) {
                            return Err(format!(
                                "Duplicate plugin entry: {} (plugin was '{}')",
                                key, plugin_entry.name
                            ));
                        }

                        let value = match arg.get(1) {
                            Some(&v) => v,
                            None => {
                                return Err(format!(
                                    "Invalid plugin call: Missing value for argument `{}`",
                                    key
                                ));
                            }
                        }
                        .to_string();

                        plugin_entry.args.insert(key, value);
                    }
                }

                plugins.push(plugin_entry);
            }
            "--lang" => {
                language = match keyval.get(1) {
                    Some(v) => match EmitLang::from_ext(v) {
                        Some(v) => Some(v),
                        None => {
                            return Err(format!(
                                "Expected language identifier to be one of: {}\nGot: '{}'",
                                "'ts', 'tsx', 'rs'", v
                            ))
                        }
                    },
                    None => {
                        return Err(String::from(
                            "Expected language identifier \
                        for output file",
                        ))
                    }
                };
            }
            "--ignore-out-naming" => ignore_naming_convention = true,
            _ => return Err(format!("Unrecognized option: '{}'", key)),
        }
    }

    if schema_file.is_none() {
        return Err(String::from("Missing option --schema"));
    }
    if out_file.is_none() {
        return Err(String::from("Missing option --out-file"));
    }
    if language.is_none() {
        // Try to detect language from the file extension
        let out_file_path = out_file.as_ref().unwrap();
        let ext = out_file_path.split(".").last();
        let _lang = match ext {
            Some(v) => match EmitLang::from_ext(v) {
                Some(v) => Some(v),
                None => None,
            },
            None => None,
        };
        if _lang.is_none() {
            return Err(String::from(
                "Could not guess language from file extension.\
            Manually provide a language with --lang.",
            ));
        } else {
            language = _lang;
        }
    }

    if field_casing.is_some() && untagged_repr {
        return Err(String::from(
            "Can not use --field-casing and --untagged together. Responses will always be camelCase.",
        ));
    }

    // Check naming convention for out_file
    if !ignore_naming_convention
        && !out_file.clone().unwrap().contains("-generated")
        && !out_file.clone().unwrap().contains("_generated")
    {
        return Err(String::from(
            "The output filename does not contain the string '_generated' or \
            '-generated'. Please add any one substring to the filename or use \
            the '--ignore-out-naming' flag to disable this check",
        ));
    }

    Ok(ZetroArgs {
        language: language.unwrap(),
        out_file: out_file.unwrap(),
        schema_file: schema_file.unwrap(),
        mangle: should_mangle,
        untagged: untagged_repr,
        field_casing,
        plugins,
    })
}
