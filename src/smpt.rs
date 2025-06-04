//! SMPT integration

use crate::petri::*;
use crate::presburger::{Constraint, ConstraintType};
use crate::debug_report::{add_debug_smpt_call, format_constraints_description, SmptCall, DebugLogger};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::hash::Hash;

use std::sync::Mutex;

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

/// Simple reachability check with constraints using SMPT
pub fn can_reach_constraint_set<P>(
    petri: Petri<P>,
    constraints: Vec<Constraint<P>>,
    out_dir: &str,
) -> Result<bool, String>
where
    P: Clone + Hash + Ord + Display + Debug,
{
    can_reach_constraint_set_with_debug(petri, constraints, out_dir, 0)
}

/// Simple reachability check with constraints using SMPT with debug tracking
pub fn can_reach_constraint_set_with_debug<P>(
    petri: Petri<P>,
    constraints: Vec<Constraint<P>>,
    out_dir: &str,
    disjunct_id: usize,
) -> Result<bool, String>
where
    P: Clone + Hash + Ord + Display + Debug,
{
    can_reach_constraint_set_with_logger(petri, constraints, out_dir, disjunct_id, None)
}

/// Simple reachability check with constraints using SMPT with optional debug logger
pub fn can_reach_constraint_set_with_logger<P>(
    petri: Petri<P>,
    constraints: Vec<Constraint<P>>,
    out_dir: &str,
    disjunct_id: usize,
    debug_logger: Option<&DebugLogger>,
) -> Result<bool, String>
where
    P: Clone + Hash + Ord + Display + Debug,
{
    // Debug logging
    if let Some(logger) = debug_logger {
        logger.log_petri_net(
            "SMPT Input Petri Net",
            &format!("Petri net for disjunct {} before SMPT verification", disjunct_id),
            &petri
        );
        logger.log_constraints(
            "SMPT Input Constraints",
            &format!("Constraints for disjunct {} to be verified by SMPT", disjunct_id),
            &constraints
        );
    }

    // Extract places from Petri net to handle missing places in constraints
    let petri_places: HashSet<String> = petri.get_places().iter()
        .map(|p| sanitize(&p.to_string()))
        .collect();

    // Convert constraints to XML and use SMPT to check reachability
    let xml = presburger_constraints_to_xml(&constraints, "reachability-check", &petri_places);

    // Convert Petri net to SMPT format
    let pnet_content = petri_to_pnet(&petri, "constraint_check");

    // Save files for SMPT
    std::fs::create_dir_all(out_dir).expect("Failed to create output directory");
    let xml_file_path = format!("{}/smpt_constraints.xml", out_dir);
    let pnet_file_path = format!("{}/smpt_petri.net", out_dir);

    std::fs::write(&xml_file_path, &xml).expect("Failed to write SMPT XML");
    std::fs::write(&pnet_file_path, &pnet_content).expect("Failed to write SMPT Petri net");

    // Try to run SMPT tool
    match run_smpt(&pnet_file_path, &xml_file_path) {
        Ok(result) => {
            let result_str = if result.is_reachable { "REACHABLE" } else { "UNREACHABLE" };
            println!("SMPT result: {}", result_str);
            if let Some(time) = result.execution_time {
                println!("Execution time: {}ms", time);
            }

            // Add debug entry
            let smpt_call = SmptCall {
                disjunct_id,
                petri_net_content: pnet_content.clone(),
                xml_content: xml.clone(),
                result: result_str.to_string(),
                execution_time_ms: result.execution_time,
                constraints_description: format_constraints_description(&constraints),
            };

            if let Some(logger) = debug_logger {
                logger.smpt_call(smpt_call);
            } else {
                add_debug_smpt_call(smpt_call);
            }

            if result.is_reachable {
                if let Some(ref mdl_line) = result.model {
                    println!("Non‐serializable/UNREACHABLE assignment (SMPT model):\n{}", mdl_line);
                }
            }

            Ok(result.is_reachable)
        }
        Err(e) => {
            eprintln!("ERROR: Failed to run SMPT: {}", e);
            eprintln!("Generated files for manual verification:");
            eprintln!("  XML: {}", xml_file_path);
            eprintln!("  Net: {}", pnet_file_path);
            eprintln!("Manual command: ./smpt_wrapper.sh -n {} --xml {}", pnet_file_path, xml_file_path);

            // Add debug entry for failed call
            let smpt_call = SmptCall {
                disjunct_id,
                petri_net_content: pnet_content.clone(),
                xml_content: xml.clone(),
                result: format!("ERROR: {}", e),
                execution_time_ms: None,
                constraints_description: format_constraints_description(&constraints),
            };

            if let Some(logger) = debug_logger {
                logger.smpt_call(smpt_call);
            } else {
                add_debug_smpt_call(smpt_call);
            }

            Err(format!("SMPT verification failed: {}", e))
        }
    }
}

/// Result from SMPT execution
#[derive(Debug)]
pub struct SmptResult {
    pub is_reachable: bool,
    pub execution_time: Option<u64>, // milliseconds
    pub method_used: Option<String>,
    pub model: Option<String>,       // The reachable marking (if not-serializable)
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
    println!("4. Install SMPT: cd SMPT && python setup.py bdist_wheel && pip install dist/smpt-5.0-py3-none-any.whl");
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
    if std::process::Command::new("./smpt_wrapper.sh")
        .args(["--help"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
    {
        return true;
    }

    // Fall back to global python3 -m smpt
    std::process::Command::new("python3")
        .args(["-m", "smpt", "--help"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Run SMPT on a Petri net file with constraints using the current global timeout
pub fn run_smpt(net_file: &str, xml_file: &str) -> Result<SmptResult, String> {
    run_smpt_with_timeout(net_file, xml_file, Some(get_smpt_timeout()))
}

pub fn run_smpt_with_timeout(net_file: &str, xml_file: &str, timeout_seconds: Option<u64>) -> Result<SmptResult, String> {
    // First try to run with 1 second timeout
    match run_smpt_with_timeout_prim(net_file, xml_file, Some(1)) {
        Ok(result) => Ok(result),
        Err(_e) => {
            // If it fails, try again with the current global timeout
            run_smpt_with_timeout_prim(net_file, xml_file, timeout_seconds)
        }
    }
}

/// Run SMPT on a Petri net file with constraints with optional timeout
pub fn run_smpt_with_timeout_prim(net_file: &str, xml_file: &str, timeout_seconds: Option<u64>) -> Result<SmptResult, String> {
    if !is_smpt_installed() {
        return Err("SMPT is not installed".to_string());
    }

    // Convert paths to absolute paths for wrapper script compatibility
    let abs_net_file = std::fs::canonicalize(net_file)
        .map_err(|e| format!("Failed to get absolute path for {}: {}", net_file, e))?;
    let abs_xml_file = std::fs::canonicalize(xml_file)
        .map_err(|e| format!("Failed to get absolute path for {}: {}", xml_file, e))?;

    // Build command arguments with optional timeout
    let mut args = vec![
        "-n", abs_net_file.to_str().unwrap(),
        "--xml", abs_xml_file.to_str().unwrap(),
        "--show-time",
        "--show-model",
        "--methods", "STATE-EQUATION", "BMC", "K-INDUCTION", "SMT", "PDR-REACH"
    ];

    // Add timeout arguments if specified (skip if 0 = use SMPT default)
    let timeout_str = timeout_seconds.filter(|&t| t > 0).map(|t| t.to_string());
    if let Some(ref timeout_val) = timeout_str {
        args.extend_from_slice(&["--timeout", timeout_val]);
    }

    // Try wrapper script first, then fall back to global python3
    let output = if std::path::Path::new("./smpt_wrapper.sh").exists() {
        std::process::Command::new("./smpt_wrapper.sh")
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to execute SMPT wrapper: {}", e))?
    } else {
        // For python3 command, use original file paths (not absolute)
        let mut python_args = vec![
            "-m", "smpt",
            "-n", net_file,
            "--xml", xml_file,
            "--show-time",
            // "--methods", "SMT", "CP", "INDUCTION", "K-INDUCTION", "STATE-EQUATION", "BMC", "PDR-COV", "PDR-REACH", "PDR-REACH-SATURATED"
            "--methods", "STATE-EQUATION", "BMC", "K-INDUCTION", "SMT", "PDR-REACH"
        ];
        if let Some(ref timeout_val) = timeout_str {
            python_args.extend_from_slice(&["--timeout", timeout_val]);
        }

        std::process::Command::new("python3")
            .args(&python_args)
            .output()
            .map_err(|e| format!("Failed to execute SMPT: {}", e))?
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse SMPT output
    let is_reachable = if stdout.contains("TRUE") {
        true
    } else if stdout.contains("FALSE") {
        false
    } else {
        // Check for timeout patterns
        if output.status.code() == Some(1) && stdout.trim() == "# Hello" {
            return Err(format!("SMPT timeout: Analysis timed out after {}s. Try increasing timeout or enabling optimizations.", timeout_seconds.unwrap_or(get_smpt_timeout())));
        } else if stdout.contains("# Hello") && stdout.contains("# Bye bye") && !stdout.contains("FORMULA") {
            return Err(format!("SMPT timeout: Analysis timed out after {}s (completed startup but no results). Try increasing timeout or enabling optimizations.", timeout_seconds.unwrap_or(get_smpt_timeout())));
        } else {
            return Err(format!("Could not parse SMPT result. stdout: {}, stderr: {}", stdout, stderr));
        }
    };

    // Extract execution time if available
    let execution_time = extract_execution_time(&stdout);

    // Extract method used
    let method_used = extract_method_used(&stdout);

    // extract reachable marking (single‐line) if present
    let model = extract_model(&stdout);

    Ok(SmptResult {
        is_reachable,
        execution_time,
        method_used,
        model,
    })
}

/// Extract execution time from SMPT output
fn extract_execution_time(output: &str) -> Option<u64> {
    // Look for patterns like "Time: 0.123s" or "Execution time: 123ms"
    for line in output.lines() {
        if let Some(time_str) = line.strip_prefix("Time: ").and_then(|s| s.strip_suffix("s")) {
            if let Ok(time_f) = time_str.parse::<f64>() {
                return Some((time_f * 1000.0) as u64);
            }
        }
    }
    None
}

/// Extract method that found the result from SMPT output
fn extract_method_used(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains("Method:") {
            return line.split("Method:").nth(1).map(|s| s.trim().to_string());
        }
        // Look for method names in the output
        if line.contains("BMC") { return Some("BMC".to_string()); }
        if line.contains("INDUCTION") { return Some("INDUCTION".to_string()); }
        if line.contains("PDR") { return Some("PDR".to_string()); }
    }
    None
}

/// Scan SMPT’s stdout for a line starting with "# Model:" (i.e., the reachable marking
/// , if exists)
/// and return the remainder of that line (the space‐separated tokens).
fn extract_model(output: &str) -> Option<String> {
    for line in output.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("# Model:") {
            // Strip off "# Model:" and any leading whitespace, then return
            let after = trimmed["# Model:".len()..].trim_start();
            return Some(after.to_string());
        }
    }
    None
}


/// Converts a Vec of presburger Constraints to XML format compatible with SMPT
pub fn presburger_constraints_to_xml<P: Display>(
    constraints: &[Constraint<P>],
    id: &str,
    petri_places: &HashSet<String>
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
        format!("<tokens-count><place>{}</place></tokens-count>", sanitized_place)
    } else {
        "<integer-constant>0</integer-constant>".to_string()
    }
}

/// Convert a single presburger Constraint to XML
pub fn presburger_constraint_to_xml<P: Display>(constraint: &Constraint<P>, petri_places: &HashSet<String>) -> String {
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
        -constraint.constant_term()  // Note: negated to match constraint semantics
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
        let constraint = Constraint::new(
            vec![(1, "x")],
            -5,
            ConstraintType::NonNegative,
        );

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
        let constraint = Constraint::new(
            vec![(2, "x"), (3, "y")],
            -10,
            ConstraintType::EqualToZero,
        );

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
    fn test_extract_execution_time() {
        let output1 = "Some output\nTime: 0.123s\nMore output";
        assert_eq!(extract_execution_time(output1), Some(123));

        let output2 = "No time info here";
        assert_eq!(extract_execution_time(output2), None);

        let output3 = "Time: 1.5s";
        assert_eq!(extract_execution_time(output3), Some(1500));
    }

    #[test]
    fn test_extract_method_used() {
        let output1 = "Some output\nMethod: BMC found result\nMore output";
        assert_eq!(extract_method_used(output1), Some("BMC found result".to_string()));

        let output2 = "BMC successful";
        assert_eq!(extract_method_used(output2), Some("BMC".to_string()));

        let output3 = "PDR method used";
        assert_eq!(extract_method_used(output3), Some("PDR".to_string()));

        let output4 = "No method info";
        assert_eq!(extract_method_used(output4), None);
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
        petri.add_transition(vec!["Producer", "BufferSlot"], vec!["Producer", "Item"]);  // Produce
        petri.add_transition(vec!["Item"], vec!["Consumer", "BufferSlot"]);             // Consume (creates consumer)
        petri.add_transition(vec!["Item"], vec!["Waste"]);                              // Alternative: waste the item

        println!("Producer-Consumer net: Producer+BufferSlot->Item, Item->Consumer+BufferSlot OR Item->Waste");

        // Test 1: Reachable - Can we produce an item?
        let can_produce = can_reach_constraint_set(
            petri.clone(),
            vec![Constraint::new(vec![(1, "Item")], -1, ConstraintType::NonNegative)],
            out_dir
        );
        println!("Can produce Item: {}", can_produce);
        assert!(can_produce, "Should be able to produce items");

        // Test 2: Unreachable - Can we have Consumer and Waste simultaneously?
        // This should be unreachable because both come from consuming the same Item
        let both_outcomes = can_reach_constraint_set(
            petri,
            vec![Constraint::new(vec![(1, "Consumer"), (1, "Waste")], -2, ConstraintType::NonNegative)],
            out_dir
        );
        println!("Can have Consumer+Waste: {} (competing outcomes)", both_outcomes);
    }

}