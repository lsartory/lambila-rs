use lambila::{VhdlVersion, lex_file, parse_file};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 || args.len() > 5 {
        eprintln!("Usage:");
        eprintln!("  {} <lex|parse> <vhdl-file> [version]", args[0]);
        eprintln!(
            "  {} fmt <input-vhdl-file> <output-vhdl-file> [version]",
            args[0]
        );
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  lex    Tokenise the file and print each token");
        eprintln!("  parse  Parse the file and print the AST");
        eprintln!("  fmt    Format the input file and save it to the output file");
        eprintln!();
        eprintln!("Versions: 1987, 1993 (default), 2008");
        process::exit(1);
    }

    let command = &args[1];
    let _path = &args[2];

    let is_fmt = command.as_str() == "fmt";

    let version_index = if is_fmt { 4 } else { 3 };
    let version_arg = args.get(version_index);

    let version = if let Some(ver) = version_arg {
        match ver.as_str() {
            "1987" | "87" => VhdlVersion::Vhdl1987,
            "1993" | "93" => VhdlVersion::Vhdl1993,
            "2008" | "08" => VhdlVersion::Vhdl2008,
            other => {
                eprintln!("Unknown VHDL version '{}'. Use 1987, 1993, or 2008.", other);
                process::exit(1);
            }
        }
    } else {
        VhdlVersion::Vhdl1993
    };

    match command.as_str() {
        "lex" => run_lex(&args[2], version),
        "parse" => run_parse(&args[2], version),
        "fmt" => {
            if args.len() < 4 {
                eprintln!("Error: 'fmt' command requires an output file path.");
                process::exit(1);
            }
            run_fmt(&args[2], &args[3], version)
        }
        other => {
            eprintln!("Unknown command '{}'. Use 'lex', 'parse', or 'fmt'.", other);
            process::exit(1);
        }
    }
}

fn run_lex(path: &str, version: VhdlVersion) {
    let result = match lex_file(path, version) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error reading '{}': {}", path, e);
            process::exit(1);
        }
    };

    for token in &result.tokens {
        println!(
            "{:>4}:{:<3}  {:20}  {}",
            token.span.line,
            token.span.col,
            format!("{:?}", token.kind),
            token.text
        );
    }

    if !result.errors.is_empty() {
        eprintln!();
        eprintln!("--- {} error(s) ---", result.errors.len());
        for err in &result.errors {
            eprintln!("  {}", err);
        }
        process::exit(2);
    }
}

fn run_parse(path: &str, version: VhdlVersion) {
    let result = match parse_file(path, version) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error reading '{}': {}", path, e);
            process::exit(1);
        }
    };

    println!("{:#?}", result.design_file);

    if !result.errors.is_empty() {
        eprintln!();
        eprintln!("--- {} parse error(s) ---", result.errors.len());
        for err in &result.errors {
            eprintln!("  [{}:{}] {}", err.span.line, err.span.col, err.message);
        }
        process::exit(2);
    }
}

fn run_fmt(input_path: &str, output_path: &str, version: VhdlVersion) {
    if let Err(e) = lambila::export_file(input_path, output_path, version) {
        eprintln!(
            "Error formatting '{}' to '{}': {}",
            input_path, output_path, e
        );
        process::exit(1);
    }
    println!(
        "Successfully formatted '{}' to '{}'",
        input_path, output_path
    );
}
