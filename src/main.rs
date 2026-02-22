use clap::Parser;
use lambila_rs::VhdlProject;
use std::fs;
use std::path::{Path, PathBuf};

/// VHDL Hierarchy parsed extraction framework
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Recursively search through directories for .vhd and .vhdl files
    #[arg(short, long)]
    recursive: bool,

    /// VHDL file or directory paths to parse
    #[arg(required = true)]
    paths: Vec<PathBuf>,
}

fn parse_path(project: &mut VhdlProject, path: &Path, recursive: bool) {
    if path.is_dir() {
        if !recursive {
            eprintln!("Skipping directory {} (use -r to recurse)", path.display());
            return;
        }
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                parse_path(project, &entry.path(), recursive);
            }
        }
    } else if path.is_file()
        && let Some(ext) = path.extension().map(|ext| ext.to_ascii_lowercase())
        && (ext == "vhd" || ext == "vhdl")
        && let Err(e) = project.parse_file(path.to_str().unwrap())
    {
        eprintln!("Error opening or parsing file {}: {}", path.display(), e);
        std::process::exit(1);
    }
}

fn main() {
    let cli = Cli::parse();
    let mut project = VhdlProject::new();

    for path in &cli.paths {
        parse_path(&mut project, path, cli.recursive);
    }

    project.print_hierarchy();
}
