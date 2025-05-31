#![allow(dead_code)]

mod affine_constraints;
mod debug_report;
mod expr_to_ns;
mod graphviz;
mod isl;
mod kleene;
mod ns;
mod ns_to_petri;
mod parser;
mod petri;
mod presburger;
mod reachability;
mod semilinear;
mod smpt;
mod spresburger;

use colored::*;
use parser::Program;
use parser::Request;
use std::env;
use std::fmt::Display;
use std::fs;
use std::hash::Hash;
use std::path::Path;
use std::process;

use ns::NS;
use parser::{ExprHc, parse, parse_program};

fn print_usage() {
    println!("{}", "Usage: ser [options] <filename or directory>".bold());
    println!("{}", "Options:".bold());
    println!(
        "  {}                  Open generated visualization files",
        "--open".green()
    );
    println!(
        "  {}               Check SMPT installation status",
        "--check-smpt".green()
    );
    println!(
        "  {}      Set SMPT timeout in seconds (default: 300)",
        "--timeout <seconds>".green()
    );
    println!("");
    println!("  - {}", "If a file is provided:".bold());
    println!(
        "    - {}: Parses as a Network System (NS), saves as graphviz, converts to Petri net and saves that as graphviz and .net",
        ".json extension".yellow()
    );
    println!(
        "    - {}: Parses as an Expr, converts to NS, and processes it like json files",
        ".ser extension".yellow()
    );
    println!("  - {}", "If a directory is provided:".bold());
    println!(
        "    - Recursively processes all {} and {} files in the directory and its subdirectories",
        ".json".yellow(),
        ".ser".yellow()
    );
    println!("  - {}:", "Output".bold());
    println!(
        "    - GraphViz ({}, {}) visualizations for Network Systems and Petri nets",
        ".dot".yellow(),
        ".png".yellow()
    );
    println!(
        "    - Petri net files ({}) in the same directory structure as GraphViz files",
        ".net".yellow()
    );
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
            }
            "--check-smpt" => {
                smpt::ensure_smpt_available();
                process::exit(0);
            }
            "--timeout" => {
                if i + 1 >= args.len() {
                    eprintln!(
                        "{}: --timeout requires a value",
                        "Error".red().bold()
                    );
                    print_usage();
                    process::exit(1);
                }
                i += 1;
                match args[i].parse::<u64>() {
                    Ok(timeout) => {
                        smpt::set_smpt_timeout(timeout);
                        println!("Set SMPT timeout to {} seconds", timeout);
                        i += 1;
                    }
                    Err(_) => {
                        eprintln!(
                            "{}: Invalid timeout value '{}'",
                            "Error".red().bold(),
                            args[i]
                        );
                        print_usage();
                        process::exit(1);
                    }
                }
            }
            _ => {
                // If it's not a recognized flag, it must be the path
                if path_str.is_empty() {
                    path_str = &args[i];
                    i += 1;
                } else {
                    // We already have a path, so this is an error
                    eprintln!(
                        "{}: Unexpected argument '{}'",
                        "Error".red().bold(),
                        args[i]
                    );
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
        eprintln!("{}: '{}' does not exist", "Error".red().bold(), path_str);
        process::exit(1);
    }

    if path.is_dir() {
        // Process directory recursively
        match process_directory(path, open_files) {
            Ok(count) => {
                println!(
                    "{} {} files",
                    "Successfully processed".green().bold(),
                    count
                );
            }
            Err(err) => {
                eprintln!("{} directory: {}", "Error processing".red().bold(), err);
                process::exit(1);
            }
        }
    } else {
        // Process single file
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => process_json_file(path_str, open_files),
            Some("ser") => process_ser_file(path_str, open_files),
            _ => {
                eprintln!(
                    "{}: Unsupported file extension for '{}'. Please use {} or {}",
                    "Error".red().bold(),
                    path_str,
                    ".json".yellow(),
                    ".ser".yellow()
                );
                print_usage();
                process::exit(1);
            }
        }
    }
}

// Process a Network System: generate visualizations for NS, Petri net, and Petri net with requests
fn process_ns<G, L, Req, Resp>(ns: &NS<G, L, Req, Resp>, out_dir: &str, open_files: bool)
where
    G: Clone + Ord + Hash + Display + std::fmt::Debug,
    L: Clone + Ord + Hash + Display + std::fmt::Debug,
    Req: Clone + Ord + Hash + Display + std::fmt::Debug,
    Resp: Clone + Ord + Hash + Display + std::fmt::Debug,
{
    // Create the output directory if it doesn't exist
    match fs::create_dir_all("out") {
        Ok(_) => {}
        Err(err) => {
            eprintln!(
                "{} output directory: {}",
                "Failed to create".red().bold(),
                err
            );
            process::exit(1);
        }
    }

    // Generate GraphViz output for the Network System
    println!("{}", "Generating GraphViz visualization...".cyan().bold());

    match ns.save_graphviz(&out_dir, open_files) {
        Ok(files) => {
            println!(
                "{} the following Network System files:",
                "Successfully generated".green().bold()
            );
            for file in files {
                println!("- {}", file.green());
            }
        }
        Err(err) => {
            eprintln!(
                "{} NS visualization: {}",
                "Failed to save".red().bold(),
                err
            );
            process::exit(1);
        }
    }

    // Convert to Petri net
    println!(
        "{}",
        "Converting to Petri net and generating visualization..."
            .cyan()
            .bold()
    );
    let petri = ns_to_petri::ns_to_petri(ns);

    // Generate Petri net visualization
    match petri.save_graphviz(&out_dir, open_files) {
        Ok(files) => {
            println!(
                "{} the following Petri net files:",
                "Successfully generated".green().bold()
            );
            for file in files {
                println!("- {}", file.green());
            }
        }
        Err(err) => {
            eprintln!(
                "{} Petri net visualization: {}",
                "Failed to save".red().bold(),
                err
            );
            process::exit(1);
        }
    }

    // Output Petri net in .net format
    let pnet_content = petri.to_pnet(out_dir);
    let pnet_file = format!("{}/petri.net", out_dir);
    match fs::create_dir_all(format!("{}", out_dir)) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{} directory: {}", "Failed to create".red().bold(), err);
            process::exit(1);
        }
    }
    match fs::write(&pnet_file, pnet_content) {
        Ok(_) => println!("- {}", pnet_file.green()),
        Err(err) => {
            eprintln!(
                "{} Petri net in .net format: {}",
                "Failed to save".red().bold(),
                err
            );
            process::exit(1);
        }
    }

    // Convert to Petri net with requests
    println!(
        "{}",
        "Converting to Petri net with requests and generating visualization..."
            .cyan()
            .bold()
    );
    let petri_with_requests = ns_to_petri::ns_to_petri_with_requests(ns);

    // Use the same output directory for Petri net with requests
    // Create a custom method or modify the underlying implementation to use a different viz_type
    // For now, we need to make a direct call to the graphviz module
    let dot_content = petri_with_requests.to_graphviz();
    match crate::graphviz::save_graphviz(&dot_content, &out_dir, "petri_with_requests", open_files)
    {
        Ok(files) => {
            println!(
                "{} the following Petri net with requests files:",
                "Successfully generated".green().bold()
            );
            for file in files {
                println!("- {}", file.green());
            }
        }
        Err(err) => {
            eprintln!(
                "{} Petri net with requests visualization: {}",
                "Failed to save".red().bold(),
                err
            );
            process::exit(1);
        }
    }

    // Output Petri net with requests in .net format
    let pnet_req_content = petri_with_requests.to_pnet(&format!("{}_with_requests", out_dir));
    let pnet_req_file = format!("{}/petri_with_requests.net", out_dir);
    match fs::write(&pnet_req_file, pnet_req_content) {
        Ok(_) => println!("- {}", pnet_req_file.green()),
        Err(err) => {
            eprintln!(
                "{} Petri net with requests in .net format: {}",
                "Failed to save".red().bold(),
                err
            );
            process::exit(1);
        }
    }

    // Output the Regex to semilinear.txt
    let regex = ns.serialized_automaton_regex();
    let regex_file = format!("{}/semilinear.txt", out_dir);
    let mut regex_content = String::new();
    regex_content.push_str(&format!("Regex: {}\n", regex));
    regex_content.push_str(&format!(
        "Semilinear:\n{}\n",
        ns.serialized_automaton_semilinear()
    ));
    match fs::write(&regex_file, regex_content) {
        Ok(_) => println!("- {}", regex_file.green()),
        Err(err) => {
            eprintln!(
                "{} Regex in semilinear format: {}",
                "Failed to save".red().bold(),
                err
            );
            process::exit(1);
        }
    }

    // Check serializability
    println!("{}", "Checking serializability...".cyan().bold());
    let serializable = ns.is_serializable(&out_dir);
    println!(
        "Serializable: {}",
        if serializable {
            "Yes".green().bold()
        } else {
            "No".red().bold()
        }
    );
}

fn process_json_file(file_path: &str, open_files: bool) {
    println!("{} {}", "Processing JSON file:".blue().bold(), file_path);

    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("{} file: {}", "Error reading".red().bold(), err);
            process::exit(1);
        }
    };

    // Parse the JSON as a Network System
    let ns = match NS::<String, String, String, String>::from_json(&content) {
        Ok(ns) => ns,
        Err(err) => {
            eprintln!(
                "{} JSON as Network System: {}",
                "Error parsing".red().bold(),
                err
            );
            process::exit(1);
        }
    };

    // Get the file name without extension to use as the base name for output files
    let path = Path::new(file_path);
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("network");
    let out_dir = format!("out/{}", file_stem);

    // Process the Network System
    process_ns(&ns, &out_dir, open_files);
}

fn process_ser_file(file_path: &str, open_files: bool) {
    println!(
        "{}",
        "----------------------------------------".blue().bold()
    );
    println!("{} {}", "Processing Ser file:".blue().bold(), file_path);

    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("{} file: {}", "Error reading".red().bold(), err);
            process::exit(1);
        }
    };

    // Try to parse as a program with multiple requests first
    let mut table = ExprHc::new();
    let ns = match parse_program(&content, &mut table) {
        Ok(program) => {
            println!(
                "{} {} requests",
                "Parsed program with".blue().bold(),
                program.requests.len()
            );
            // Convert program to Network System
            println!(
                "{}",
                "Converting program to Network System...".cyan().bold()
            );
            expr_to_ns::program_to_ns(&mut table, &program)
        }
        Err(_) => {
            // Fall back to parsing as a single expression
            match parse(&content, &mut table) {
                Ok(expr) => {
                    println!("{} {}", "Parsed expression:".blue().bold(), expr);
                    // Convert expression to Network System
                    println!(
                        "{}",
                        "Converting expression to Network System...".cyan().bold()
                    );
                    expr_to_ns::program_to_ns(
                        &mut table,
                        &Program {
                            requests: vec![Request {
                                name: "request".to_string(),
                                body: expr,
                            }],
                        },
                    )
                }
                Err(err) => {
                    eprintln!("{} SER file: {}", "Error parsing".red().bold(), err);
                    process::exit(1);
                }
            }
        }
    };

    // Get the file name without extension to use as the base name for output files
    let path = Path::new(file_path);
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("expr");
    let out_dir = format!("out/{}", file_stem);

    // Process the Network System
    process_ns(&ns, &out_dir, open_files);
}

// Recursively process all files in a directory and its subdirectories
fn process_directory(dir: &Path, open_files: bool) -> Result<usize, String> {
    let mut processed_count = 0;

    // Read directory contents
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            return Err(format!(
                "{} directory '{}': {}",
                "Error reading".red().bold(),
                dir.display(),
                err
            ));
        }
    };

    // Process each entry
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!(
                    "{}: Error accessing entry: {}",
                    "Warning".yellow().bold(),
                    err
                );
                continue;
            }
        };

        let path = entry.path();

        if path.is_dir() {
            // Recursively process subdirectory
            match process_directory(&path, open_files) {
                Ok(count) => processed_count += count,
                Err(err) => eprintln!("{}: {}", "Warning".yellow().bold(), err),
            }
        } else if path.is_file() {
            // Process file if it has a supported extension
            if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
                let path_str = path.to_string_lossy().to_string();

                match ext {
                    "json" => {
                        process_json_file(&path_str, open_files);
                        processed_count += 1;
                    }
                    "ser" => {
                        process_ser_file(&path_str, open_files);
                        processed_count += 1;
                    }
                    _ => {} // Skip files with unsupported extensions
                }
                println!("");
            }
        }
    }

    Ok(processed_count)
}
