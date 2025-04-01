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

impl Constraints {
    /// Adds constraints that each specified place must equal zero,
    /// but only if such a constraint doesn't already exist in each AND clause
    pub fn assert_places_zero(&mut self, places: &[Var]) {
        for and_clause in &mut self.constraints {
            for place in places {
                // Check if this exact zero constraint already exists in this AND clause
                let already_constrained = and_clause.iter().any(|constraint| {
                    // Exact match for "place = 0" constraint
                        constraint.affine_formula == vec![(1, *place)] &&
                        constraint.offset == 0 &&
                         constraint.constraint_type == EqualToZero
                });

                if !already_constrained {
                    and_clause.push(Constraint {
                        affine_formula: vec![(1, *place)],
                        offset: 0,
                        constraint_type: EqualToZero,
                    });
                }
            }
        }
    }
}

impl Constraints {
    /// Extracts all variables from a clause that have constraints of the form "coeff*var = 0"
    /// (EqualToZero with single variable and zero offset, any coefficient)
    pub fn extract_zero_variables(clause: &[Constraint]) -> Vec<Var> {
        let mut zero_vars = Vec::new();

        for constraint in clause {
            if constraint.constraint_type == EqualToZero &&
                constraint.affine_formula.len() == 1 &&
                constraint.offset == 0
            {
                zero_vars.push(constraint.affine_formula[0].1);
            }
        }

        zero_vars
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


#[test]
pub fn test_extract_zero_variables() {
    let clause = vec![
        Constraint {
            affine_formula: vec![(1, Var(0))],  // 1*P0 = 0
            offset: 0,
            constraint_type: EqualToZero,
        },
        Constraint {
            affine_formula: vec![(2, Var(1))],  // 2*P1 = 0
            offset: 0,
            constraint_type: EqualToZero,
        },
        Constraint {
            affine_formula: vec![(-3, Var(2))],  // -3*P2 = 0
            offset: 0,
            constraint_type: EqualToZero,
        },
        Constraint {
            affine_formula: vec![(1, Var(3)), (-1, Var(4))],  // P3 - P4 = 0
            offset: 0,
            constraint_type: EqualToZero,  // Doesn't match - multiple variables
        },
        Constraint {
            affine_formula: vec![(1, Var(5))],
            offset: 1,  // P5 + 1 = 0
            constraint_type: EqualToZero,  // Doesn't match - non-zero offset
        },
        Constraint {
            affine_formula: vec![(1, Var(6))],
            offset: 0,
            constraint_type: NonNegative,  // Doesn't match - wrong type
        },
    ];

    let zero_vars = Constraints::extract_zero_variables(&clause);
    assert_eq!(zero_vars, vec![Var(0), Var(1), Var(2)]);
}
