// #![allow(dead_code)]

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
    println!("Usage: ser [options] <filename or directory>");
    println!("Options:");
    println!("  --open                  Open generated visualization files");
    println!("");
    println!("  - If a file is provided:");
    println!("    - .json extension: Parses as a Network System (NS), saves as graphviz, converts to Petri net and saves that as graphviz");
    println!("    - .ser extension: Parses as an Expr, converts to NS, and processes it like json files");
    println!("  - If a directory is provided:");
    println!("    - Recursively processes all .json and .ser files in the directory and its subdirectories");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse command line flags
    let mut open_files = false;
    let mut path_str = "";

    // Skip the program name (args[0])
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--open" => {
                open_files = true;
                i += 1;
            },
            _ => {
                // If it's not a recognized flag, it must be the path
                if path_str.is_empty() {
                    path_str = &args[i];
                    i += 1;
                } else {
                    // We already have a path, so this is an error
                    eprintln!("Error: Unexpected argument '{}'", args[i]);
                    print_usage();
                    process::exit(1);
                }
            }
        }
    }

    // Ensure we have a path
    if path_str.is_empty() {
        print_usage();
        process::exit(1);
    }

    let path = Path::new(path_str);

    if !path.exists() {
        eprintln!("Error: '{}' does not exist", path_str);
        process::exit(1);
    }

    if path.is_dir() {
        // Process directory recursively
        match process_directory(path, open_files) {
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
            Some("json") => process_json_file(path_str, open_files),
            Some("ser") => process_ser_file(path_str, open_files),
            _ => {
                eprintln!("Error: Unsupported file extension for '{}'. Please use .json or .ser", path_str);
                print_usage();
                process::exit(1);
            }
        }
    }
}

// Process a Network System: generate visualizations for NS, Petri net, and Petri net with requests
fn process_ns<G, L, Req, Resp>(ns: &NS<G, L, Req, Resp>, file_stem: &str, open_files: bool)
where
    G: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Display,
    L: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Display,
    Req: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Display,
    Resp: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Display,
{
    // Generate GraphViz output for the Network System
    println!("Generating GraphViz visualization...");

    match ns.save_graphviz(file_stem, open_files) {
        Ok(files) => {
            println!("Successfully generated the following Network System files:");
            for file in files {
                println!("- {}", file);
            }
        },
        Err(err) => {
            eprintln!("Failed to save NS visualization: {}", err);
            process::exit(1);
        }
    }

    // Convert to Petri net
    println!("Converting to Petri net and generating visualization...");
    let petri = ns_to_petri::ns_to_petri(ns);

    // Generate Petri net visualization
    match petri.save_graphviz(file_stem, open_files) {
        Ok(files) => {
            println!("Successfully generated the following Petri net files:");
            for file in files {
                println!("- {}", file);
            }
        },
        Err(err) => {
            eprintln!("Failed to save Petri net visualization: {}", err);
            process::exit(1);
        }
    }

    // Convert to Petri net with requests
    println!("Converting to Petri net with requests and generating visualization...");
    let petri_with_requests = ns_to_petri::ns_to_petri_with_requests(ns);

    // Use the same output directory for Petri net with requests
    // Create a custom method or modify the underlying implementation to use a different viz_type
    // For now, we need to make a direct call to the graphviz module
    let dot_content = petri_with_requests.to_graphviz();
    match crate::graphviz::save_graphviz(&dot_content, file_stem, "petri_with_requests", open_files) {
        Ok(files) => {
            println!("Successfully generated the following Petri net with requests files:");
            for file in files {
                println!("- {}", file);
            }
        },
        Err(err) => {
            eprintln!("Failed to save Petri net with requests visualization: {}", err);
            process::exit(1);
        }
    }
}

fn process_json_file(file_path: &str, open_files: bool) {
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

    // Get the file name without extension to use as the base name for output files
    let path = Path::new(file_path);
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("network");

    // Process the Network System
    process_ns(&ns, file_stem, open_files);
}

fn process_ser_file(file_path: &str, open_files: bool) {
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
    let expr = match parse(&content, &mut table) {
        Ok(expr) => {
            println!("Parsed expression: {}", expr);
            expr
        },
        Err(err) => {
            eprintln!("Error parsing SER file: {}", err);
            process::exit(1);
        }
    };

    // Convert expression to Network System
    println!("Converting expression to Network System...");
    let ns = expr_to_ns::expr_to_ns(&mut table, &expr);

    // Get the file name without extension to use as the base name for output files
    let path = Path::new(file_path);
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("expr");

    // Process the Network System
    process_ns(&ns, file_stem, open_files);
}

// Recursively process all files in a directory and its subdirectories
fn process_directory(dir: &Path, open_files: bool) -> Result<usize, String> {
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
            match process_directory(&path, open_files) {
                Ok(count) => processed_count += count,
                Err(err) => eprintln!("Warning: {}", err),
            }
        } else if path.is_file() {
            // Process file if it has a supported extension
            if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
                let path_str = path.to_string_lossy().to_string();

                match ext {
                    "json" => {
                        process_json_file(&path_str, open_files);
                        processed_count += 1;
                    },
                    "ser" => {
                        process_ser_file(&path_str, open_files);
                        processed_count += 1;
                    },
                    _ => {} // Skip files with unsupported extensions
                }
            }
        }
    }

    Ok(processed_count)
}