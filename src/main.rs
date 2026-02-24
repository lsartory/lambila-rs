use lambila::{VhdlVersion, lex_file};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} <vhdl-file> [version]", args[0]);
        eprintln!();
        eprintln!("Versions: 1987, 1993 (default), 2008");
        process::exit(1);
    }

    let path = &args[1];

    let version = if args.len() == 3 {
        match args[2].as_str() {
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
