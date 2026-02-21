use lambila_rs::VhdlFile;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    match VhdlFile::open(file_path) {
        Ok(vhdl_file) => {
            vhdl_file.print_entities();
        }
        Err(e) => {
            eprintln!("Error opening or parsing file {}: {}", file_path, e);
            std::process::exit(1);
        }
    }
}
