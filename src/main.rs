use lambila::{VhdlVersion, lex_file, parse_file};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 || args.len() > 4 {
        eprintln!("Usage: {} <lex|parse> <vhdl-file> [version]", args[0]);
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  lex    Tokenise the file and print each token");
        eprintln!("  parse  Parse the file and print the AST");
        eprintln!();
        eprintln!("Versions: 1987, 1993 (default), 2008");
        process::exit(1);
    }

    let command = &args[1];
    let path = &args[2];

    let version = if args.len() == 4 {
        match args[3].as_str() {
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
        "lex" => run_lex(path, version),
        "parse" => run_parse(path, version),
        other => {
            eprintln!("Unknown command '{}'. Use 'lex' or 'parse'.", other);
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
