use lambila::{VhdlVersion, lex_file, parse};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_usage(&args[0]);
        process::exit(1);
    }

    let command = &args[1];
    let path = &args[2];

    let version = if args.len() >= 4 {
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
        "lex" => cmd_lex(path, version),
        "parse" => cmd_parse(path, version),
        "format" => cmd_format(path, version),
        other => {
            eprintln!("Unknown command '{}'. Use lex, parse, or format.", other);
            process::exit(1);
        }
    }
}

fn print_usage(program: &str) {
    eprintln!("Usage: {} <command> <vhdl-file> [version]", program);
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  lex     Lex the file and print tokens");
    eprintln!("  parse   Lex and parse the file, then print the AST");
    eprintln!("  format  Lex and parse the file, re-export formatted VHDL");
    eprintln!();
    eprintln!("Versions: 1987, 1993 (default), 2008");
}

fn lex_or_exit(path: &str, version: VhdlVersion) -> lambila::LexResult {
    match lex_file(path, version) {
        Ok(r) => {
            if !r.errors.is_empty() {
                eprintln!("--- {} lexer error(s) ---", r.errors.len());
                for err in &r.errors {
                    eprintln!("  {}", err);
                }
            }
            r
        }
        Err(e) => {
            eprintln!("Error reading '{}': {}", path, e);
            process::exit(1);
        }
    }
}

fn cmd_lex(path: &str, version: VhdlVersion) {
    let result = lex_or_exit(path, version);

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
        process::exit(2);
    }
}

fn cmd_parse(path: &str, version: VhdlVersion) {
    let result = lex_or_exit(path, version);

    match parse(&result.tokens) {
        Ok(design_file) => {
            println!("{:#?}", design_file);
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            process::exit(2);
        }
    }
}

fn cmd_format(path: &str, version: VhdlVersion) {
    let result = lex_or_exit(path, version);

    match parse(&result.tokens) {
        Ok(design_file) => {
            print!("{}", design_file);
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            process::exit(2);
        }
    }
}
