//! Affine constraints, like might be output from ISL
//!
//! Variables are normalized to be v0, v1, v2, ...

use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Var(pub usize);

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "P{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ConstraintType {
    NonNegative,
    EqualToZero,
}
pub use ConstraintType::*;

#[derive(Debug, Clone)]
pub struct Constraint {
    // Represents  \sum_i a_i x_i + b >= 0 or \sum_i a_i x_i + b = 0 where a_i are the coefficients and b is the offset
    /// Linear combination of variables: (coeff, var) pairs
    pub affine_formula: Vec<(i32, Var)>,
    pub offset: i32,
    /// What kind of constraint (inequality or equality)
    pub constraint_type: ConstraintType,
}

impl Constraint {
    pub fn is_redundant(&self) -> bool {
        self.affine_formula.iter().all(|(coeff, _)| *coeff >= 0)
            && self.offset >= 0
            && self.constraint_type == NonNegative
    }
}

/// Variables 0...N-1 are the real variables.
/// Variables N...N+k-1 are the newly introduced existential variables
///
/// All variables have a domain of $\mathbb{N}$, but the constants / coefficients can be negative.
#[derive(Debug, Clone)]
pub struct Constraints {
    pub num_vars: usize,             // N
    pub num_existential_vars: usize, // k

    /// A big OR over a bunch of big ANDs of constraints
    pub constraints: Vec<Vec<Constraint>>,
}

impl PartialEq for Constraints {
    fn eq(&self, other: &Self) -> bool {
        // Convert both constraints to ISL sets and check equality using ISL
        use crate::isl;

        let set1 = isl::affine_constraints_to_isl_set(self);
        let set2 = isl::affine_constraints_to_isl_set(other);

        // Check equality using ISL
        let result = unsafe {
            let is_equal = isl::isl_set_is_equal(set1, set2);

            // Clean up
            isl::isl_set_free(set1);
            isl::isl_set_free(set2);

            is_equal != 0
        };

        result
    }
}

impl Eq for Constraints {}

impl Constraints {
    pub fn new(
        num_vars: usize,
        num_existential_vars: usize,
        mut constraints: Vec<Vec<Constraint>>,
    ) -> Self {
        // Eliminate redundant constraints of the form (positive linear combination) >= 0
        for constraint_vec in constraints.iter_mut() {
            constraint_vec.retain(|constraint| !constraint.is_redundant());
        }
        Self {
            num_vars,
            num_existential_vars,
            constraints,
        }
    }

    /// Add this constraint as a conjunct.
    ///
    /// Because the constraints are in DNF, this adds it to every disjunct in the list.
    pub fn assert(&mut self, conjunct: Constraint) {
        if conjunct.is_redundant() {
            return;
        }
        for c in &mut self.constraints {
            c.push(conjunct.clone());
        }
    }
}

// Converts a full Constraints structure to XML with proper nesting
pub fn constraints_to_xml(constraints: &Constraints, id: &str) -> String {
    let mut xml = format!(
        r#"<?xml version='1.0' encoding='utf-8'?>
<property-set>
  <property>
    <id>{}</id>
    <description>Generated from affine constraints</description>
    <formula>
      <exists-path>
        <finally>
          <disjunction>
"#,
        id
    );

    // Each top-level group is a disjunct
    for and_clause in &constraints.constraints {
        // If we have multiple constraints in a group, wrap in conjunction
        if and_clause.len() > 1 {
            xml.push_str("            <conjunction>\n");
        }

        // Add each constraint in the group
        for constraint in and_clause {
            let constraint_xml = single_constraint_to_xml(constraint);
            for line in constraint_xml.lines() {
                if and_clause.len() > 1 {
                    xml.push_str("              ");
                } else {
                    xml.push_str("            ");
                }
                xml.push_str(line);
                xml.push_str("\n");
            }
        }

        if and_clause.len() > 1 {
            xml.push_str("            </conjunction>\n");
        }
    }

    xml.push_str(
        r#"          </disjunction>
        </finally>
      </exists-path>
    </formula>
  </property>
</property-set>"#,
    );

    xml
}

// Keep the existing constraint_to_xml function as is
pub fn single_constraint_to_xml(constraint: &Constraint) -> String {
    let mut xml = String::new();

    let operator = match constraint.constraint_type {
        NonNegative => "integer-ge",
        EqualToZero => "integer-eq",
    };

    xml.push_str(&format!("<{}>\n", operator));

    // Build the affine expression
    if constraint.affine_formula.len() == 1 && constraint.affine_formula[0].0 == 1 {
        xml.push_str(&format!(
            "  <tokens-count><place>{}</place></tokens-count>\n",
            constraint.affine_formula[0].1
        ));
    } else {
        xml.push_str("  <integer-add>\n");
        for (coeff, var) in &constraint.affine_formula {
            if *coeff == 1 {
                xml.push_str(&format!(
                    "    <tokens-count><place>{var}</place></tokens-count>\n"
                ));
            } else {
                xml.push_str("    <integer-mul>\n");
                xml.push_str(&format!(
                    "      <integer-constant>{}</integer-constant>\n",
                    coeff
                ));
                xml.push_str(&format!(
                    "      <tokens-count><place>{var}</place></tokens-count>\n"
                ));
                xml.push_str("    </integer-mul>\n");
            }
        }
        xml.push_str("  </integer-add>\n");
    }

    // Add the offset
    if constraint.offset != 0 {
        xml.push_str(&format!(
            "  <integer-constant>{}</integer-constant>\n",
            -constraint.offset
        ));
    } else {
        xml.push_str("  <integer-constant>0</integer-constant>\n");
    }

    xml.push_str(&format!("</{}>", operator));
    xml
}

#[test]
pub fn test_to_xml_1() {
    // Create example constraints matching the XML example
    // (2P0 + P1 ≥ 4) OR (P0 = P1)
    let constraints = Constraints {
        num_vars: 2, // P0 (v0) and P1 (v1)
        num_existential_vars: 0,
        constraints: vec![
            // First OR clause: 2P0 + P1 ≥ 4
            vec![Constraint {
                affine_formula: vec![(2, Var(0)), (1, Var(1))],
                offset: -4, // 2P0 + P1 - 4 ≥ 0
                constraint_type: NonNegative,
            }],
            // Second OR clause: P0 = P1
            vec![Constraint {
                affine_formula: vec![(1, Var(0)), (-1, Var(1))],
                offset: 0,
                constraint_type: EqualToZero,
            }],
        ],
    };

    let xml = constraints_to_xml(&constraints, "test-1-true");
    println!("{}", xml);

    // Verify the output contains expected XML fragments
    assert!(xml.contains("<disjunction>"));
    assert!(xml.contains("<integer-ge>"));
    assert!(xml.contains("<integer-eq>"));
    assert!(xml.contains("<place>P0</place>"));
    assert!(xml.contains("<place>P1</place>"));
    assert!(xml.contains("<integer-constant>2</integer-constant>"));
    assert!(xml.contains("<integer-constant>4</integer-constant>"));
}

#[test]
pub fn test_to_xml_2() {
    // Create example constraints matching the XML example
    // (2P0 + P1 ≥ 4) AND (P0 = P1)
    let constraints = Constraints {
        num_vars: 2, // P1 (v0) and P2 (v1)
        num_existential_vars: 0,
        constraints: vec![vec![
            Constraint {
                //  2P0 + P1 ≥ 4
                affine_formula: vec![(2, Var(0)), (1, Var(1))],
                offset: -4, // 2P0 + P1 - 4 ≥ 0
                constraint_type: NonNegative,
            },
            Constraint {
                // P0 = P1
                affine_formula: vec![(1, Var(0)), (-1, Var(1))],
                offset: 0,
                constraint_type: EqualToZero,
            },
        ]],
    };

    let xml = constraints_to_xml(&constraints, "test-2-false");
    println!("{}", xml);

    // Verify the output contains expected XML fragments
    assert!(xml.contains("<disjunction>"));
    assert!(xml.contains("<integer-ge>"));
    assert!(xml.contains("<integer-eq>"));
    assert!(xml.contains("<place>P0</place>"));
    assert!(xml.contains("<place>P1</place>"));
    assert!(xml.contains("<integer-constant>2</integer-constant>"));
    assert!(xml.contains("<integer-constant>4</integer-constant>"));
}
