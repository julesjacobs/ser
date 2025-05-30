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
    let xml_file_path = format!("{}/smpt_constraints.xml", out_dir);
    let pnet_file_path = format!("{}/smpt_petri.net", out_dir);
    
    std::fs::write(&xml_file_path, &xml).expect("Failed to write SMPT XML");
    std::fs::write(&pnet_file_path, &pnet_content).expect("Failed to write SMPT Petri net");
    
    // TODO: Actually run SMPT tool here
    // For now, return false (conservative)
    println!("Checking reachability with {} constraints on {} places", 
             constraints.len(), petri.get_places().len());
    println!("Generated files:");
    println!("  XML: {}", xml_file_path);
    println!("  Net: {}", pnet_file_path);
    false
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
}