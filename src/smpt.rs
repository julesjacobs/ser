//! SMPT (Satisfiability Modulo Petri Nets) integration module.
//!
//! This module provides Rust bindings to the SMPT tool for verifying
//! reachability properties in Petri nets. It supports:
//!
//! - Converting Petri nets to SMPT's .net format
//! - Converting Presburger constraints to SMPT's XML format
//! - Running SMPT with configurable timeouts and retry logic
//! - Parsing results including proofs and counterexample traces
//!
//! # Examples
//! ```
//! use smpt::{can_reach_constraint_set, SmptOptions};
//!
//! let result = can_reach_constraint_set(
//!     petri,
//!     constraints,
//!     "out/",
//!     SmptOptions::default()
//! )?;
//! ```

use crate::debug_report::{SmptCall, format_constraints_description};
use crate::petri::*;
use crate::presburger::{Constraint, ConstraintType};
use crate::proof_parser::{ProofInvariant, parse_proof_file};
use colored::*;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::Mutex;

// === Constants ===
const SMPT_WRAPPER_PATH: &str = "./smpt_wrapper.sh";
const SMPT_PYTHON_MODULE: &str = "smpt";
const DEFAULT_METHODS: &[&str] = &["STATE-EQUATION", "BMC", "K-INDUCTION", "SMT", "PDR-REACH"];

// === Result Types ===

/// Inner verification result type
#[derive(Debug, Clone)]
pub enum SmptVerificationOutcome {
    /// The constraint set is unreachable (program is serializable)
    Unreachable {
        /// Proof certificate if available
        proof_certificate: Option<String>,
        /// Parsed proof invariant if parsing succeeded
        parsed_proof: Option<ProofInvariant<String>>,
    },
    /// The constraint set is reachable (program is not serializable)
    Reachable {
        /// Counterexample trace (sequence of transition indices from the Petri net)
        trace: Vec<usize>,
    },
    /// Verification failed or timed out
    Error { message: String },
}

/// Result of SMPT verification with proof/counterexample information and raw output
#[derive(Debug, Clone)]
pub struct SmptVerificationResult {
    /// The verification outcome
    pub outcome: SmptVerificationOutcome,
    /// Raw SMPT stdout for debugging
    pub raw_stdout: String,
    /// Raw SMPT stderr for debugging
    pub raw_stderr: String,
}

// === Configuration ===
/// Default SMPT timeout in seconds (can be configured at runtime)
///
/// **Configuration:** To change the timeout globally, use `set_smpt_timeout()`:
/// - `2` = 2 seconds (default, good for quick testing)
/// - `60` = 1 minute (for most examples)
/// - `300` = 5 minutes (for complex examples)
/// - `600` = 10 minutes (for very complex examples)
/// - `0` = Use SMPT's default timeout (225 seconds)
///
/// This timeout is passed to SMPT's `--timeout` argument, which limits
/// execution time per property verification method.
static SMPT_TIMEOUT_SECONDS: Mutex<u64> = Mutex::new(10);

/// Get the current SMPT timeout value
pub fn get_smpt_timeout() -> u64 {
    *SMPT_TIMEOUT_SECONDS.lock().unwrap()
}

/// Set the global SMPT timeout value
pub fn set_smpt_timeout(timeout_seconds: u64) {
    *SMPT_TIMEOUT_SECONDS.lock().unwrap() = timeout_seconds;
}

// === Public Types ===

/// Convert a Petri net to SMPT .net format
/// Produces a textual representation of the Petri net compatible with SMPT tools
pub fn petri_to_pnet<Place>(petri: &Petri<Place>, net_name: &str) -> String
where
    Place: ToString + Clone + PartialEq + Eq + Hash,
{
    // A small helper to sanitize non-alphanumeric chars from strings.
    fn sanitize(s: &str) -> String {
        s.chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect()
    }

    let mut out = String::new();

    // 1. net {...}
    out.push_str(&format!("net {{{}}}\n", sanitize(net_name)));

    // 2. Count how many times each place appears in the initial marking.
    let mut marking_count: HashMap<String, usize> = HashMap::new();
    for place in petri.get_initial_marking() {
        let place_str = sanitize(&place.to_string());
        *marking_count.entry(place_str).or_insert(0) += 1;
    }

    // 3. Output the "pl" lines, e.g. "pl P1 (1)"
    //    for each place in initial marking.
    for (place, count) in marking_count {
        out.push_str(&format!("pl {} ({})\n", place, count));
    }

    // 4. Output each transition, named t0, t1, ...
    for (i, (input_places, output_places)) in petri.get_transitions().iter().enumerate() {
        // "tr tX <inputs> -> <outputs>"
        out.push_str(&format!("tr t{} ", i));

        // Input places
        for p in input_places {
            out.push_str(&sanitize(&p.to_string()));
            out.push(' ');
        }

        // Arrow
        out.push_str("-> ");

        // Output places
        let mut first = true;
        for p in output_places {
            if !first {
                out.push(' ');
            }
            out.push_str(&sanitize(&p.to_string()));
            first = false;
        }
        out.push('\n');
    }

    out
}

// === Main API Functions ===

/// Check if constraints are reachable in a Petri net using SMPT
/// Returns detailed verification result with proof/counterexample
pub fn can_reach_constraint_set<P>(
    petri: Petri<P>,
    constraints: Vec<Constraint<P>>,
    out_dir: &str,
    disjunct_id: usize,
) -> SmptVerificationResult
where
    P: Clone + Hash + Ord + Display + Debug,
{
    // Get debug logger from global state
    let debug_logger = crate::reachability::get_debug_logger();

    // Debug logging
    debug_logger.log_petri_net(
        "SMPT Input Petri Net",
        &format!(
            "Petri net for disjunct {} before SMPT verification",
            disjunct_id
        ),
        &petri,
    );
    debug_logger.log_constraints(
        "SMPT Input Constraints",
        &format!(
            "Constraints for disjunct {} to be verified by SMPT",
            disjunct_id
        ),
        &constraints,
    );

    // Extract places from Petri net to handle missing places in constraints
    let petri_places: HashSet<String> = petri
        .get_places()
        .iter()
        .map(|p| sanitize(&p.to_string()))
        .collect();

    // Convert constraints to XML and use SMPT to check reachability
    let xml = presburger_constraints_to_xml(&constraints, "reachability-check", &petri_places);

    // Convert Petri net to SMPT format
    let pnet_content = petri_to_pnet(&petri, "constraint_check");

    // Save files for SMPT
    std::fs::create_dir_all(out_dir).expect("Failed to create output directory");
    let xml_file_path = format!("{}/smpt_constraints_disjunct_{}.xml", out_dir, disjunct_id);
    let pnet_file_path = format!("{}/smpt_petri_disjunct_{}.net", out_dir, disjunct_id);
    let _proof_file_path = format!(
        "{}/smpt_constraints_disjunct_{}_proof.txt",
        out_dir, disjunct_id
    );

    std::fs::write(&xml_file_path, &xml).expect("Failed to write SMPT XML");
    std::fs::write(&pnet_file_path, &pnet_content).expect("Failed to write SMPT Petri net");

    // Try to run SMPT tool
    let result = run_smpt(&pnet_file_path, &xml_file_path);

    // Log the result
    match &result.outcome {
        SmptVerificationOutcome::Unreachable { .. } => {
            println!(
                "  {} SMPT result: {}",
                "→".bright_black(),
                "UNREACHABLE".bright_black()
            );
        }
        SmptVerificationOutcome::Reachable { .. } => {
            println!(
                "  {} SMPT result: {}",
                "→".bright_black(),
                "REACHABLE".yellow().bold()
            );
        }
        SmptVerificationOutcome::Error { message } => {
            eprintln!("ERROR: Failed to run SMPT: {}", message);
            eprintln!("Generated files for manual verification:");
            eprintln!("  XML: {}", xml_file_path);
            eprintln!("  Net: {}", pnet_file_path);
            eprintln!(
                "Manual command: ./smpt_wrapper.sh -n {} --xml {}",
                pnet_file_path, xml_file_path
            );
        }
    }

    // Save raw SMPT output to files if available
    // For now, we need to re-run SMPT to get raw output since we removed SmptResult
    // In a future refactoring, we could include raw output in the verification result

    // Add debug entry
    let result_str = match &result.outcome {
        SmptVerificationOutcome::Unreachable { .. } => "UNREACHABLE",
        SmptVerificationOutcome::Reachable { .. } => "REACHABLE",
        SmptVerificationOutcome::Error { message } => message.as_str(),
    };

    let smpt_call = SmptCall {
        disjunct_id,
        petri_net_content: pnet_content,
        xml_content: xml,
        result: result_str.to_string(),
        execution_time_ms: None, // We measure time externally now
        constraints_description: format_constraints_description(&constraints),
    };
    debug_logger.smpt_call(smpt_call);

    // Save raw SMPT output for debugging
    let stdout_path = format!("{}/smpt_output_disjunct_{}.stdout", out_dir, disjunct_id);
    let stderr_path = format!("{}/smpt_output_disjunct_{}.stderr", out_dir, disjunct_id);
    std::fs::write(&stdout_path, &result.raw_stdout).ok();
    std::fs::write(&stderr_path, &result.raw_stderr).ok();

    result
}

/// Install SMPT tool - returns true if already installed or successfully installed
pub fn install_smpt() -> Result<(), String> {
    // Check if SMPT is already available
    if is_smpt_installed() {
        return Ok(());
    }

    println!("SMPT not found. Installation instructions:");
    println!("1. Install Python 3.7+ and pip");
    println!("2. Install Z3: pip install z3-solver");
    println!("3. Clone SMPT: git clone https://github.com/nicolasAmat/SMPT.git");
    println!(
        "4. Install SMPT: cd SMPT && python setup.py bdist_wheel && pip install dist/smpt-5.0-py3-none-any.whl"
    );
    println!("5. Alternative: pip install smpt");

    Err("SMPT is not installed. Please follow the installation instructions above.".to_string())
}

/// Check and install SMPT if needed, with user-friendly output
pub fn ensure_smpt_available() -> bool {
    if is_smpt_installed() {
        println!("✓ SMPT is available");
        return true;
    }

    println!("⚠ SMPT is not installed or not available in PATH");
    match install_smpt() {
        Ok(_) => {
            println!("✓ SMPT installation check complete");
            true
        }
        Err(msg) => {
            println!("✗ {}", msg);
            false
        }
    }
}

/// Check if SMPT is installed and available
pub fn is_smpt_installed() -> bool {
    // Try the wrapper script first
    if Path::new(SMPT_WRAPPER_PATH).exists()
        && Command::new(SMPT_WRAPPER_PATH)
            .args(["--help"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    {
        return true;
    }

    // Fall back to global python3 -m smpt
    Command::new("python3")
        .args(["-m", SMPT_PYTHON_MODULE, "--help"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Run SMPT on a Petri net file with constraints using the current global timeout
fn run_smpt(net_file: &str, xml_file: &str) -> SmptVerificationResult {
    run_smpt_with_timeout(net_file, xml_file, Some(get_smpt_timeout()))
}

/// Run SMPT with a specific timeout
fn run_smpt_with_timeout(
    net_file: &str,
    xml_file: &str,
    timeout_seconds: Option<u64>,
) -> SmptVerificationResult {
    run_smpt_internal(net_file, xml_file, timeout_seconds)
}

// === Helper Functions ===

/// Build SMPT command arguments
fn build_smpt_args(
    net_file: &str,
    xml_file: &str,
    _proof_file: &str,
    timeout_seconds: Option<u64>,
) -> Vec<String> {
    let mut args = vec![
        "-n".to_string(),
        net_file.to_string(),
        "--xml".to_string(),
        xml_file.to_string(),
        "--show-time".to_string(),
        "--show-model".to_string(),
        "--debug".to_string(),
        // TODO: Temporarily disable proof flags to enable trace output
        // "--export-proof".to_string(),
        // proof_file.to_string(),
    ];

    // Add methods
    args.push("--methods".to_string());
    for method in DEFAULT_METHODS {
        args.push(method.to_string());
    }

    // Add timeout if specified
    if let Some(timeout) = timeout_seconds.filter(|&t| t > 0) {
        args.push("--timeout".to_string());
        args.push(timeout.to_string());
    }

    args
}

/// Execute SMPT command
fn execute_smpt(args: &[String]) -> Result<Output, std::io::Error> {
    // Try wrapper script first
    if Path::new(SMPT_WRAPPER_PATH).exists() {
        Command::new(SMPT_WRAPPER_PATH).args(args).output()
    } else {
        // Fall back to python3 -m smpt
        let mut python_args = vec!["-m".to_string(), SMPT_PYTHON_MODULE.to_string()];
        python_args.extend_from_slice(args);

        Command::new("python3").args(&python_args).output()
    }
}

/// Extract model from SMPT output
fn extract_model(output: &str) -> Option<String> {
    for line in output.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("# Model:") {
            let after = trimmed["# Model:".len()..].trim_start();
            return Some(after.to_string());
        }
    }
    None
}

/// Extract trace from SMPT output, converting transition IDs to indices
/// Looks for traces in the output itself or in associated .scn files
fn extract_trace(output: &str) -> Vec<usize> {
    let lines: Vec<&str> = output.lines().collect();

    // First, look for traces in the output itself (traditional format)
    for i in 0..lines.len() {
        // Look for BMC or PDR trace markers
        if lines[i].contains("[BMC] Trace") || lines[i].contains("[PDR] Trace") {
            // Next non-empty line should contain the trace
            if i + 1 < lines.len() {
                let trace_line = lines[i + 1].trim();
                if !trace_line.is_empty() && trace_line.starts_with('t') {
                    return trace_line
                        .split_whitespace()
                        .filter_map(|s| {
                            // Extract number from "t0", "t1", etc.
                            s.strip_prefix('t')
                                .and_then(|num| num.parse::<usize>().ok())
                        })
                        .collect();
                }
            }
        }
    }

    Vec::new()
}

/// Run SMPT on a Petri net file with constraints with optional timeout (internal implementation)
fn run_smpt_internal(
    net_file: &str,
    xml_file: &str,
    timeout_seconds: Option<u64>,
) -> SmptVerificationResult {
    if !is_smpt_installed() {
        return SmptVerificationResult {
            outcome: SmptVerificationOutcome::Error {
                message: "SMPT is not installed".to_string(),
            },
            raw_stdout: String::new(),
            raw_stderr: String::new(),
        };
    }

    // Convert paths to absolute paths for wrapper script compatibility
    let abs_net_file = match std::fs::canonicalize(net_file) {
        Ok(path) => path,
        Err(e) => {
            return SmptVerificationResult {
                outcome: SmptVerificationOutcome::Error {
                    message: format!("Failed to get absolute path for {}: {}", net_file, e),
                },
                raw_stdout: String::new(),
                raw_stderr: String::new(),
            };
        }
    };
    let abs_xml_file = match std::fs::canonicalize(xml_file) {
        Ok(path) => path,
        Err(e) => {
            return SmptVerificationResult {
                outcome: SmptVerificationOutcome::Error {
                    message: format!("Failed to get absolute path for {}: {}", xml_file, e),
                },
                raw_stdout: String::new(),
                raw_stderr: String::new(),
            };
        }
    };

    // Generate absolute proof file path based on the XML file path
    let proof_file_path = abs_xml_file.to_str().unwrap().replace(".xml", "_proof.txt");

    // Build command arguments
    let args = build_smpt_args(
        abs_net_file.to_str().unwrap(),
        abs_xml_file.to_str().unwrap(),
        &proof_file_path,
        timeout_seconds,
    );

    // Execute SMPT
    let output = match execute_smpt(&args) {
        Ok(output) => output,
        Err(e) => {
            return SmptVerificationResult {
                outcome: SmptVerificationOutcome::Error {
                    message: format!("Failed to execute SMPT: {}", e),
                },
                raw_stdout: String::new(),
                raw_stderr: String::new(),
            };
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    // Parse SMPT output
    if stdout.contains("TRUE") {
        // Property is reachable => NOT serializable
        let mut trace = extract_trace(&stdout);

        // If no trace found in stdout, try to read from .scn file
        if trace.is_empty() {
            let scn_file_path = proof_file_path.replace(".txt", ".txt.scn");
            if let Ok(scn_content) = std::fs::read_to_string(&scn_file_path) {
                let trace_line = scn_content.trim();
                if !trace_line.is_empty() && trace_line.starts_with('t') {
                    trace = trace_line
                        .split_whitespace()
                        .filter_map(|s| {
                            // Extract number from "t0", "t1", etc.
                            s.strip_prefix('t')
                                .and_then(|num| num.parse::<usize>().ok())
                        })
                        .collect();
                }
            }
        }

        SmptVerificationResult {
            outcome: SmptVerificationOutcome::Reachable { trace },
            raw_stdout: stdout,
            raw_stderr: stderr,
        }
    } else if stdout.contains("FALSE") {
        // Property is unreachable => IS serializable

        // Try to read proof certificate if it exists
        let proof_certificate = std::fs::read_to_string(&proof_file_path).ok();

        // Try to parse the proof certificate
        let parsed_proof =
            proof_certificate
                .as_ref()
                .and_then(|cert| match parse_proof_file(cert) {
                    Ok(proof) => Some(proof),
                    Err(e) => {
                        eprintln!("Warning: Failed to parse proof certificate: {:?}", e);
                        None
                    }
                });

        SmptVerificationResult {
            outcome: SmptVerificationOutcome::Unreachable {
                proof_certificate,
                parsed_proof,
            },
            raw_stdout: stdout,
            raw_stderr: stderr,
        }
    } else {
        // Check for timeout patterns
        let error_msg = if output.status.code() == Some(1) && stdout.trim() == "# Hello" {
            format!(
                "SMPT timeout: Analysis timed out after {}s. Try increasing timeout or enabling optimizations.",
                timeout_seconds.unwrap_or(get_smpt_timeout())
            )
        } else if stdout.contains("# Hello")
            && stdout.contains("# Bye bye")
            && !stdout.contains("FORMULA")
        {
            format!(
                "SMPT timeout: Analysis timed out after {}s (completed startup but no results). Try increasing timeout or enabling optimizations.",
                timeout_seconds.unwrap_or(get_smpt_timeout())
            )
        } else {
            format!(
                "Could not parse SMPT result. stdout: {}, stderr: {}",
                stdout, stderr
            )
        };

        SmptVerificationResult {
            outcome: SmptVerificationOutcome::Error { message: error_msg },
            raw_stdout: stdout,
            raw_stderr: stderr,
        }
    }
}

// === Conversion Functions ===

/// Converts a Vec of presburger Constraints to XML format compatible with SMPT
pub fn presburger_constraints_to_xml<P: Display>(
    constraints: &[Constraint<P>],
    id: &str,
    petri_places: &HashSet<String>,
) -> String {
    let mut xml = format!(
        r#"<?xml version='1.0' encoding='utf-8'?>
<property-set>
  <property>
    <id>{}</id>
    <description>Generated from presburger constraints</description>
    <formula>
      <exists-path>
        <finally>
          <conjunction>
"#,
        id
    );

    // If no constraints, create a tautology (always true)
    if constraints.is_empty() {
        xml.push_str(
            r#"            <integer-eq>
              <integer-constant>0</integer-constant>
              <integer-constant>0</integer-constant>
            </integer-eq>
"#,
        );
    } else {
        // Add each constraint
        for constraint in constraints {
            let constraint_xml = presburger_constraint_to_xml(constraint, petri_places);
            for line in constraint_xml.lines() {
                xml.push_str("            ");
                xml.push_str(line);
                xml.push('\n');
            }
        }
    }

    xml.push_str(
        r#"          </conjunction>
        </finally>
      </exists-path>
    </formula>
  </property>
</property-set>"#,
    );

    xml
}

// Use the shared utility function for sanitization
use crate::utils::string::sanitize;

/// Helper function to generate place token count or constant 0 if place doesn't exist
fn place_tokens_or_zero(place_name: &str, petri_places: &HashSet<String>) -> String {
    let sanitized_place = sanitize(place_name);
    if petri_places.contains(&sanitized_place) {
        format!(
            "<tokens-count><place>{}</place></tokens-count>",
            sanitized_place
        )
    } else {
        "<integer-constant>0</integer-constant>".to_string()
    }
}

/// Convert a single presburger Constraint to XML
pub fn presburger_constraint_to_xml<P: Display>(
    constraint: &Constraint<P>,
    petri_places: &HashSet<String>,
) -> String {
    let mut xml = String::new();

    let operator = match constraint.constraint_type() {
        ConstraintType::NonNegative => "integer-ge",
        ConstraintType::EqualToZero => "integer-eq",
    };

    xml.push_str(&format!("<{}>\n", operator));

    // Build the left side (linear combination)
    let linear_combo = constraint.linear_combination();
    if linear_combo.is_empty() {
        // Special case: no variables
        xml.push_str("  <integer-constant>0</integer-constant>\n");
    } else if linear_combo.len() == 1 && linear_combo[0].0 == 1 {
        // Simple case: coefficient = 1
        xml.push_str(&format!(
            "  {}\n",
            place_tokens_or_zero(&linear_combo[0].1.to_string(), petri_places)
        ));
    } else if linear_combo.len() == 1 {
        // Single variable with coefficient != 1
        let place_xml = place_tokens_or_zero(&linear_combo[0].1.to_string(), petri_places);
        if place_xml.contains("integer-constant") {
            // If place doesn't exist, result is coefficient * 0 = 0
            xml.push_str("  <integer-constant>0</integer-constant>\n");
        } else {
            xml.push_str("  <integer-mul>\n");
            xml.push_str(&format!(
                "    <integer-constant>{}</integer-constant>\n",
                linear_combo[0].0
            ));
            xml.push_str(&format!("    {}\n", place_xml));
            xml.push_str("  </integer-mul>\n");
        }
    } else {
        // Multiple variables - use integer-add
        xml.push_str("  <integer-add>      \n");
        for (coeff, var) in linear_combo {
            let place_xml = place_tokens_or_zero(&var.to_string(), petri_places);

            if *coeff == 1 {
                xml.push_str(&format!("    {}\n", place_xml));
            } else {
                xml.push_str("    <integer-mul>\n");
                xml.push_str(&format!(
                    "      <integer-constant>{}</integer-constant>\n",
                    coeff
                ));
                xml.push_str(&format!("      {}\n", place_xml));
                xml.push_str("    </integer-mul>\n");
            }
        }
        xml.push_str("  </integer-add>\n");
    }

    // Right side (constant term)
    xml.push_str(&format!(
        "  <integer-constant>{}</integer-constant>\n",
        -constraint.constant_term() // Note: negated to match constraint semantics
    ));

    xml.push_str(&format!("</{}>", operator));
    xml
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presburger::{Constraint, ConstraintType};

    #[test]
    fn test_presburger_constraint_to_xml_simple() {
        // Test: x >= 5 (represented as x - 5 >= 0)
        let constraint = Constraint::new(vec![(1, "x")], -5, ConstraintType::NonNegative);

        // Create a set of places that includes 'x'
        let mut petri_places = HashSet::new();
        petri_places.insert("x".to_string());

        let xml = presburger_constraint_to_xml(&constraint, &petri_places);

        assert!(xml.contains("<integer-ge>"));
        assert!(xml.contains("<place>x</place>"));
        assert!(xml.contains("<integer-constant>5</integer-constant>"));
    }

    #[test]
    fn test_presburger_constraint_to_xml_equality() {
        // Test: 2x + 3y = 10 (represented as 2x + 3y - 10 = 0)
        let constraint =
            Constraint::new(vec![(2, "x"), (3, "y")], -10, ConstraintType::EqualToZero);

        // Create a set of places that includes 'x' and 'y'
        let mut petri_places = HashSet::new();
        petri_places.insert("x".to_string());
        petri_places.insert("y".to_string());

        let xml = presburger_constraint_to_xml(&constraint, &petri_places);

        assert!(xml.contains("<integer-eq>"));
        assert!(xml.contains("<integer-add>"));
        assert!(xml.contains("<place>x</place>"));
        assert!(xml.contains("<place>y</place>"));
        assert!(xml.contains("<integer-constant>2</integer-constant>"));
        assert!(xml.contains("<integer-constant>3</integer-constant>"));
        assert!(xml.contains("<integer-constant>10</integer-constant>"));
    }

    #[test]
    fn test_presburger_constraints_to_xml_empty() {
        let constraints: Vec<Constraint<&str>> = vec![];
        let petri_places = HashSet::new();
        let xml = presburger_constraints_to_xml(&constraints, "test-empty", &petri_places);

        assert!(xml.contains("<conjunction>"));
        assert!(xml.contains("<integer-eq>"));
        assert!(xml.contains("<integer-constant>0</integer-constant>"));
    }

    #[test]
    fn test_presburger_constraints_to_xml_multiple() {
        let constraints = vec![
            Constraint::new(vec![(1, "x")], -5, ConstraintType::NonNegative),
            Constraint::new(vec![(1, "y")], 0, ConstraintType::EqualToZero),
        ];

        // Create a set of places that includes 'x' and 'y'
        let mut petri_places = HashSet::new();
        petri_places.insert("x".to_string());
        petri_places.insert("y".to_string());

        let xml = presburger_constraints_to_xml(&constraints, "test-multiple", &petri_places);

        assert!(xml.contains("<conjunction>"));
        assert!(xml.contains("<integer-ge>"));
        assert!(xml.contains("<integer-eq>"));
        assert!(xml.contains("<place>x</place>"));
        assert!(xml.contains("<place>y</place>"));
    }

    #[test]
    fn test_petri_to_pnet() {
        let mut petri = Petri::new(vec!["P0", "P1"]);
        petri.add_transition(vec!["P0"], vec!["P1"]);
        petri.add_transition(vec!["P1"], vec![]);

        let pnet = petri_to_pnet(&petri, "test_net");

        assert!(pnet.contains("net {test_net}"));
        assert!(pnet.contains("pl P0 (1)"));
        assert!(pnet.contains("pl P1 (1)"));
        assert!(pnet.contains("tr t0 P0 -> P1"));
        assert!(pnet.contains("tr t1 P1 ->"));
    }

    #[test]
    fn test_petri_to_pnet_empty() {
        let petri = Petri::new(Vec::<&str>::new());
        let pnet = petri_to_pnet(&petri, "empty_net");

        assert!(pnet.contains("net {empty_net}"));
        // Should have no place or transition lines
        assert!(!pnet.contains("pl "));
        assert!(!pnet.contains("tr "));
    }

    #[test]
    fn test_petri_to_pnet_sanitization() {
        let mut petri = Petri::new(vec!["P-0", "P@1"]);
        petri.add_transition(vec!["P-0"], vec!["P@1"]);

        let pnet = petri_to_pnet(&petri, "test-net@2");

        assert!(pnet.contains("net {test_net_2}"));
        assert!(pnet.contains("pl P_0 (1)"));
        assert!(pnet.contains("pl P_1 (1)"));
        assert!(pnet.contains("tr t0 P_0 -> P_1"));
    }

    #[test]
    fn test_is_smpt_installed() {
        // This test will check if SMPT is available, but won't fail if it's not installed
        let installed = is_smpt_installed();
        println!("SMPT installed: {}", installed);
        // Always pass - this is just for information
        assert!(true);
    }

    #[test]
    fn test_extract_trace() {
        let output = r#"# Hello
####################
[BMC] Trace
t1 t0 t10 t8 t12 t5 t3
####################
[PDR] Trace checking

FORMULA reachability-check TRUE TIME 0.403745174407959
# Bye bye"#;

        let trace = extract_trace(output);
        assert_eq!(trace, vec![1, 0, 10, 8, 12, 5, 3]);

        // Test empty trace
        let no_trace = "# Hello\nFORMULA reachability-check FALSE\n# Bye bye";
        assert_eq!(extract_trace(no_trace), Vec::<usize>::new());
    }

    #[test]
    fn test_install_smpt_instructions() {
        // Test that install function provides instructions when SMPT is not installed
        if !is_smpt_installed() {
            let result = install_smpt();
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("not installed"));
        }
    }

    #[test]
    fn test_proof_parsing_integration() {
        // Test that proof certificates are parsed when available
        use crate::proof_parser::Formula;

        // Create a mock proof certificate content
        let mock_proof = r#"
        (define-fun cert ((G__X_1_ Int) (RESP_read_REQ_1 Int)) Bool
            (and (>= G__X_1_ 0) (>= RESP_read_REQ_1 0)))
        "#;

        // Parse it
        match parse_proof_file(mock_proof) {
            Ok(proof) => {
                assert_eq!(proof.variables.len(), 2);
                assert!(proof.variables.contains(&"G__X_1_".to_string()));
                assert!(proof.variables.contains(&"RESP_read_REQ_1".to_string()));

                // Check that it's an And formula with constraints
                match &proof.formula {
                    Formula::And(formulas) => {
                        assert_eq!(formulas.len(), 2);
                    }
                    _ => panic!("Expected And formula"),
                }
            }
            Err(e) => panic!("Failed to parse proof: {:?}", e),
        }
    }

    #[test]
    fn test_smpt_reachability_analysis() {
        use tempfile::TempDir;

        if !is_smpt_installed() {
            println!("SMPT not available - skipping integration test");
            return;
        }

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let out_dir = temp_dir.path().to_str().unwrap();

        // Create simple but interesting Petri net: Producer-Consumer with buffer
        let mut petri = Petri::new(vec!["Producer", "BufferSlot"]);
        petri.add_transition(vec!["Producer", "BufferSlot"], vec!["Producer", "Item"]); // Produce
        petri.add_transition(vec!["Item"], vec!["Consumer", "BufferSlot"]); // Consume (creates consumer)
        petri.add_transition(vec!["Item"], vec!["Waste"]); // Alternative: waste the item

        println!(
            "Producer-Consumer net: Producer+BufferSlot->Item, Item->Consumer+BufferSlot OR Item->Waste"
        );

        // Test 1: Reachable - Can we produce an item?
        let can_produce_result = can_reach_constraint_set(
            petri.clone(),
            vec![Constraint::new(
                vec![(1, "Item")],
                -1,
                ConstraintType::NonNegative,
            )],
            out_dir,
            0, // disjunct_id
        );
        match can_produce_result.outcome {
            SmptVerificationOutcome::Reachable { trace, .. } => {
                println!("Can produce Item: Yes (reachable), trace: {:?}", trace);
            }
            SmptVerificationOutcome::Unreachable { .. } => {
                panic!("Should be able to produce items");
            }
            SmptVerificationOutcome::Error { message } => {
                panic!("SMPT error: {}", message);
            }
        }

        // Test 2: Reachable - Can we have Consumer and Waste simultaneously?
        // Yes, we can produce multiple items and send them to different outcomes
        let both_outcomes_result = can_reach_constraint_set(
            petri,
            vec![Constraint::new(
                vec![(1, "Consumer"), (1, "Waste")],
                -2,
                ConstraintType::NonNegative,
            )],
            out_dir,
            1, // disjunct_id
        );
        match both_outcomes_result.outcome {
            SmptVerificationOutcome::Reachable { trace, .. } => {
                println!(
                    "Can have Consumer+Waste: Yes (reachable), trace: {:?}",
                    trace
                );
            }
            SmptVerificationOutcome::Unreachable { .. } => {
                panic!("Should be able to have both outcomes by producing multiple items");
            }
            SmptVerificationOutcome::Error { message } => {
                panic!("SMPT error: {}", message);
            }
        }
    }
}
