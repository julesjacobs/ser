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
    
    // First, save the example files that were previously in main()
    if let Err(err) = save_example_ns() {
        eprintln!("Warning: Failed to save example NS file: {}", err);
    }
    
    if let Err(err) = save_example_expr() {
        eprintln!("Warning: Failed to save example Expr file: {}", err);
    }
    
    // Now handle normal command line usage
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

// Save example NS to JSON files
fn save_example_ns() -> Result<(), String> {
    // Create examples directory if it doesn't exist
    let examples_dir = Path::new("examples");
    if !examples_dir.exists() {
        if let Err(err) = fs::create_dir_all(examples_dir) {
            return Err(format!("Failed to create examples directory: {}", err));
        }
    }

    // Example 1: Login flow
    {
        // Create a simple network system for a login flow
        let mut ns = NS::<String, String, String, String>::new("NoSession".to_string());

        // Add requests and responses
        ns.add_request("Login".to_string(), "Start".to_string());
        ns.add_request("Query".to_string(), "LoggedIn".to_string());
        ns.add_request("Logout".to_string(), "LoggedIn".to_string());

        ns.add_response("Start".to_string(), "Welcome".to_string());
        ns.add_response("LoggedIn".to_string(), "QueryResult".to_string());
        ns.add_response("Start".to_string(), "GoodBye".to_string());

        // Add transitions
        ns.add_transition(
            "Start".to_string(),
            "NoSession".to_string(),
            "LoggedIn".to_string(),
            "ActiveSession".to_string(),
        );

        ns.add_transition(
            "LoggedIn".to_string(),
            "ActiveSession".to_string(),
            "Start".to_string(),
            "NoSession".to_string(),
        );

        // Serialize to JSON
        let json = match ns.to_json() {
            Ok(json) => json,
            Err(err) => return Err(format!("Failed to serialize login NS to JSON: {}", err)),
        };

        // Write to file
        let file_path = examples_dir.join("login_flow.json");
        if let Err(err) = fs::write(&file_path, json) {
            return Err(format!("Failed to write login JSON file: {}", err));
        }
    }

    // Example 2: Simple data flow
    {
        let mut ns = NS::<String, String, String, String>::new("Empty".to_string());
        
        // Add requests
        ns.add_request("GetData".to_string(), "Waiting".to_string());
        ns.add_request("SaveData".to_string(), "Ready".to_string());
        
        // Add responses
        ns.add_response("DataReceived".to_string(), "Success".to_string());
        ns.add_response("Ready".to_string(), "Acknowledge".to_string());
        
        // Add transitions
        ns.add_transition(
            "Waiting".to_string(),
            "Empty".to_string(),
            "DataReceived".to_string(),
            "HasData".to_string(),
        );
        
        ns.add_transition(
            "DataReceived".to_string(),
            "HasData".to_string(),
            "Ready".to_string(),
            "Empty".to_string(),
        );

        // Serialize to JSON
        let json = match ns.to_json() {
            Ok(json) => json,
            Err(err) => return Err(format!("Failed to serialize data flow NS to JSON: {}", err)),
        };

        // Write to file
        let file_path = examples_dir.join("data_flow.json");
        if let Err(err) = fs::write(&file_path, json) {
            return Err(format!("Failed to write data flow JSON file: {}", err));
        }
    }

    Ok(())
}

// Save example Expr files to .ser files
fn save_example_expr() -> Result<(), String> {
    let examples = [
        ("if_expr.ser", "if(x == 1){y := 2}else{z := 3}"),
        ("while_expr.ser", "while(x == 0){x := 1}"),
        ("seq_expr.ser", "x := 1; y := 2; z := 3"),
        ("complex_expr.ser", "if(x == 1){if(y == 2){z := 3}else{z := 4}}else{z := 5}")
    ];

    // Create examples directory if it doesn't exist
    let examples_dir = Path::new("examples");
    if !examples_dir.exists() {
        if let Err(err) = fs::create_dir_all(examples_dir) {
            return Err(format!("Failed to create examples directory: {}", err));
        }
    }

    // Write each example to a file
    for (filename, content) in examples.iter() {
        let file_path = examples_dir.join(filename);
        if let Err(err) = fs::write(&file_path, content) {
            return Err(format!("Failed to write SER file '{}': {}", filename, err));
        }
    }

    Ok(())
}

#[test]
fn test_save_examples() {
    if let Err(err) = save_example_ns() {
        panic!("Failed to save example NS: {}", err);
    }
    
    if let Err(err) = save_example_expr() {
        panic!("Failed to save example Expr: {}", err);
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