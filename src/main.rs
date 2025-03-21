#![allow(dead_code)]

mod ns;
mod parser;
mod kleene;
mod semilinear;
mod petri;
mod graphviz;
mod expr_to_ns;
mod ns_to_petri;

use std::env;
use std::fs;
use std::path::Path;
use std::process;

use parser::{ExprHc, parse};
use ns::NS;

fn print_usage() {
    println!("Usage: ser <filename or directory>");
    println!("  - If a file is provided:");
    println!("    - .json extension: Parses as a Network System (NS) and saves as graphviz");
    println!("    - .ser extension: Parses as an Expr and pretty prints it");
    println!("  - If a directory is provided:");
    println!("    - Recursively processes all .json and .ser files in the directory and its subdirectories");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Handle command line usage
    if args.len() != 2 {
        print_usage();
        process::exit(1);
    }

    let path_str = &args[1];
    let path = Path::new(path_str);

    if !path.exists() {
        eprintln!("Error: '{}' does not exist", path_str);
        process::exit(1);
    }

    if path.is_dir() {
        // Process directory recursively
        match process_directory(path) {
            Ok(count) => {
                println!("Successfully processed {} files", count);
            },
            Err(err) => {
                eprintln!("Error processing directory: {}", err);
                process::exit(1);
            }
        }
    } else {
        // Process single file
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => process_json_file(path_str),
            Some("ser") => process_ser_file(path_str),
            _ => {
                eprintln!("Error: Unsupported file extension for '{}'. Please use .json or .ser", path_str);
                print_usage();
                process::exit(1);
            }
        }
    }
}

fn process_json_file(file_path: &str) {
    println!("Processing JSON file: {}", file_path);

    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file: {}", err);
            process::exit(1);
        }
    };

    // Parse the JSON as a Network System
    let ns = match NS::<String, String, String, String>::from_json(&content) {
        Ok(ns) => ns,
        Err(err) => {
            eprintln!("Error parsing JSON as Network System: {}", err);
            process::exit(1);
        }
    };

    // Generate GraphViz output
    println!("Generating GraphViz visualization...");

    // Get the file name without extension to use as the base name for output files
    let path = Path::new(file_path);
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("network");

    match ns.save_graphviz(file_stem, true) {
        Ok(files) => {
            println!("Successfully generated the following files:");
            for file in files {
                println!("- {}", file);
            }
        },
        Err(err) => {
            eprintln!("Failed to save visualization: {}", err);
            process::exit(1);
        }
    }
}

fn process_ser_file(file_path: &str) {
    println!("Processing SER file: {}", file_path);

    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file: {}", err);
            process::exit(1);
        }
    };

    // Parse the content as an Expr
    let mut table = ExprHc::new();
    match parse(&content, &mut table) {
        Ok(expr) => {
            println!("Parsed expression: {}", expr);
        },
        Err(err) => {
            eprintln!("Error parsing SER file: {}", err);
            process::exit(1);
        }
    }
}

// Recursively process all files in a directory and its subdirectories
fn process_directory(dir: &Path) -> Result<usize, String> {
    let mut processed_count = 0;

    // Read directory contents
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => return Err(format!("Error reading directory '{}': {}", dir.display(), err)),
    };

    // Process each entry
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!("Warning: Error accessing entry: {}", err);
                continue;
            }
        };

        let path = entry.path();

        if path.is_dir() {
            // Recursively process subdirectory
            match process_directory(&path) {
                Ok(count) => processed_count += count,
                Err(err) => eprintln!("Warning: {}", err),
            }
        } else if path.is_file() {
            // Process file if it has a supported extension
            if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
                let path_str = path.to_string_lossy().to_string();

                match ext {
                    "json" => {
                        process_json_file(&path_str);
                        processed_count += 1;
                    },
                    "ser" => {
                        process_ser_file(&path_str);
                        processed_count += 1;
                    },
                    _ => {} // Skip files with unsupported extensions
                }
            }
        }
    }

    Ok(processed_count)
}