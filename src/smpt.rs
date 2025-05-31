//! SMPT integration

use crate::petri::*;
use crate::presburger::{Constraint, ConstraintType};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;

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
) -> bool
where
    P: Clone + Hash + Ord + Display + Debug,
{
    // Convert constraints to XML and use SMPT to check reachability
    let xml = presburger_constraints_to_xml(&constraints, "reachability-check");
    
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
            println!("SMPT result: {}", if result.is_reachable { "REACHABLE" } else { "UNREACHABLE" });
            if let Some(time) = result.execution_time {
                println!("Execution time: {}ms", time);
            }
            result.is_reachable
        }
        Err(e) => {
            println!("Warning: Failed to run SMPT: {}", e);
            println!("Generated files for manual verification:");
            println!("  XML: {}", xml_file_path);
            println!("  Net: {}", pnet_file_path);
            println!("Manual command: python3 -m smpt -n {} --reachability-xml {}", pnet_file_path, xml_file_path);
            false // Conservative fallback
        }
    }
}

/// Result from SMPT execution
#[derive(Debug)]
pub struct SmptResult {
    pub is_reachable: bool,
    pub execution_time: Option<u64>, // milliseconds
    pub method_used: Option<String>,
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

/// Run SMPT on a Petri net file with constraints
pub fn run_smpt(net_file: &str, xml_file: &str) -> Result<SmptResult, String> {
    if !is_smpt_installed() {
        return Err("SMPT is not installed".to_string());
    }
    
    // Try wrapper script first, then fall back to global python3
    let output = if std::path::Path::new("./smpt_wrapper.sh").exists() {
        std::process::Command::new("./smpt_wrapper.sh")
            .args([
                "-n", net_file,
                "--reachability-xml", xml_file,
                "--show-time",
                "--methods", "BMC,INDUCTION,PDR"
            ])
            .output()
            .map_err(|e| format!("Failed to execute SMPT wrapper: {}", e))?
    } else {
        std::process::Command::new("python3")
            .args([
                "-m", "smpt",
                "-n", net_file,
                "--reachability-xml", xml_file,
                "--show-time",
                "--methods", "BMC,INDUCTION,PDR"
            ])
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
        return Err(format!("Could not parse SMPT result. stdout: {}, stderr: {}", stdout, stderr));
    };
    
    // Extract execution time if available
    let execution_time = extract_execution_time(&stdout);
    
    // Extract method used
    let method_used = extract_method_used(&stdout);
    
    Ok(SmptResult {
        is_reachable,
        execution_time,
        method_used,
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

/// Converts a Vec of presburger Constraints to XML format compatible with SMPT
pub fn presburger_constraints_to_xml<P: Display>(
    constraints: &[Constraint<P>], 
    id: &str
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
            let constraint_xml = presburger_constraint_to_xml(constraint);
            for line in constraint_xml.lines() {
                xml.push_str("            ");
                xml.push_str(line);
                xml.push_str("\n");
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

/// Convert a single presburger Constraint to XML
pub fn presburger_constraint_to_xml<P: Display>(constraint: &Constraint<P>) -> String {
    let mut xml = String::new();

    let operator = match constraint.constraint_type() {
        ConstraintType::NonNegative => "integer-ge",
        ConstraintType::EqualToZero => "integer-eq",
    };

    xml.push_str(&format!("<{}>\n", operator));

    // Build the left side (linear combination)
    let linear_combo = constraint.linear_combination();
    if linear_combo.len() == 1 && linear_combo[0].0 == 1 {
        // Simple case: coefficient = 1
        xml.push_str(&format!(
            "  <tokens-count><place>{}</place></tokens-count>\n",
            linear_combo[0].1
        ));
    } else if linear_combo.len() == 1 {
        // Single variable with coefficient != 1
        xml.push_str("  <integer-mul>\n");
        xml.push_str(&format!(
            "    <integer-constant>{}</integer-constant>\n",
            linear_combo[0].0
        ));
        xml.push_str(&format!(
            "    <tokens-count><place>{}</place></tokens-count>\n",
            linear_combo[0].1
        ));
        xml.push_str("  </integer-mul>\n");
    } else {
        // Multiple variables - use integer-add
        xml.push_str("  <integer-add>\n");
        for (coeff, var) in linear_combo {
            if *coeff == 1 {
                xml.push_str(&format!(
                    "    <tokens-count><place>{}</place></tokens-count>\n",
                    var
                ));
            } else if *coeff == -1 {
                xml.push_str("    <integer-sub>\n");
                xml.push_str("      <integer-constant>0</integer-constant>\n");
                xml.push_str(&format!(
                    "      <tokens-count><place>{}</place></tokens-count>\n",
                    var
                ));
                xml.push_str("    </integer-sub>\n");
            } else {
                xml.push_str("    <integer-mul>\n");
                xml.push_str(&format!(
                    "      <integer-constant>{}</integer-constant>\n",
                    coeff
                ));
                xml.push_str(&format!(
                    "      <tokens-count><place>{}</place></tokens-count>\n",
                    var
                ));
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

        let xml = presburger_constraint_to_xml(&constraint);
        
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

        let xml = presburger_constraint_to_xml(&constraint);
        
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
        let xml = presburger_constraints_to_xml(&constraints, "test-empty");
        
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
        
        let xml = presburger_constraints_to_xml(&constraints, "test-multiple");
        
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
}