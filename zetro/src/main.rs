mod common;
mod generators;
mod utilities;

fn main() {
    use std::io::Write;

    let args = match utilities::parse_args(std::env::args().collect::<Vec<String>>()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let schema_contents =
        std::fs::read_to_string(&args.schema_file).expect("error reading schema.json");

    let schema_json = serde_json::from_str::<serde_json::Value>(&schema_contents)
        .expect("error decoding JSON from string");
    let schema = match common::schema::ZetroSchema::from_value(&schema_json) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let generated_result = match args.language {
        utilities::EmitLang::Rust => generators::rust::generate_schema_code(schema, &args),
        utilities::EmitLang::Typescript => {
            generators::typescript::generate_schema_code(schema, &args)
        }
    };

    if let Err(e) = generated_result {
        eprintln!("error generating file: {}", e);
        return;
    }
    let generated_file = generated_result.unwrap();

    let mut out_file = std::fs::File::create(args.out_file).expect("error opening output file");
    out_file
        .write_all(generated_file.as_bytes())
        .expect("error writing to output file");
}
