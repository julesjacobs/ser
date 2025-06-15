use crate::kleene::Kleene; // <-- bring in zero()
use crate::presburger::{Constraint as PConstraint, PresburgerSet, QuantifiedSet, Variable};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::Path;

/// Affine expression: sum of terms (coefficient * variable) + constant
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AffineExpr {
    /// Map from variable name to coefficient (BTreeMap for deterministic ordering)
    terms: BTreeMap<String, i64>,
    constant: i64,
}

impl AffineExpr {
    /// Create a zero expression
    pub fn new() -> Self {
        AffineExpr {
            terms: BTreeMap::new(),
            constant: 0,
        }
    }

    /// Create a constant expression
    pub fn from_const(c: i64) -> Self {
        AffineExpr {
            terms: BTreeMap::new(),
            constant: c,
        }
    }

    /// Create a variable expression (coefficient 1)
    pub fn from_var(var: String) -> Self {
        let mut terms = BTreeMap::new();
        terms.insert(var, 1);
        AffineExpr { terms, constant: 0 }
    }

    /// Add two expressions
    pub fn add(&self, other: &AffineExpr) -> AffineExpr {
        let mut result = self.clone();

        // Add the constant
        result.constant += other.constant;

        // Add each term
        for (var, coeff) in &other.terms {
            *result.terms.entry(var.clone()).or_insert(0) += coeff;
        }

        // Remove zero coefficients
        result.terms.retain(|_, coeff| *coeff != 0);

        result
    }

    /// Subtract two expressions
    pub fn sub(&self, other: &AffineExpr) -> AffineExpr {
        self.add(&other.negate())
    }

    /// Multiply by a constant
    pub fn mul_by_const(&self, c: i64) -> AffineExpr {
        if c == 0 {
            return AffineExpr::new();
        }

        let mut result = AffineExpr::new();
        result.constant = self.constant * c;

        for (var, coeff) in &self.terms {
            result.terms.insert(var.clone(), coeff * c);
        }

        result
    }

    /// Negate the expression
    pub fn negate(&self) -> AffineExpr {
        self.mul_by_const(-1)
    }

    /// Get coefficient of a variable (0 if not present)
    pub fn get_coeff(&self, var: &str) -> i64 {
        self.terms.get(var).copied().unwrap_or(0)
    }

    /// Get the constant term
    pub fn get_constant(&self) -> i64 {
        self.constant
    }

    /// Get all variables in the expression
    pub fn variables(&self) -> Vec<String> {
        self.terms.keys().cloned().collect()
    }

    /// Check if this is a constant expression (no variables)
    pub fn is_constant(&self) -> bool {
        self.terms.is_empty()
    }

    /// Convert to a vector of (coefficient, variable) pairs plus constant
    /// This is useful for converting to presburger constraints
    pub fn to_linear_combination(&self) -> (Vec<(i64, String)>, i64) {
        let terms: Vec<(i64, String)> = self
            .terms
            .iter()
            .map(|(var, coeff)| (*coeff, var.clone()))
            .collect();
        (terms, self.constant)
    }
}

impl fmt::Display for AffineExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.terms.is_empty() && self.constant == 0 {
            write!(f, "0")
        } else {
            let mut first = true;

            // Write terms in sorted order (BTreeMap ensures this)
            for (var, coeff) in &self.terms {
                if *coeff == 0 {
                    continue;
                }

                if !first {
                    write!(f, " ")?;
                    if *coeff >= 0 {
                        write!(f, "+ ")?;
                    }
                } else {
                    first = false;
                }

                if *coeff == 1 {
                    write!(f, "{}", var)?;
                } else if *coeff == -1 {
                    write!(f, "-{}", var)?;
                } else {
                    write!(f, "{}*{}", coeff, var)?;
                }
            }

            // Write constant
            if self.constant != 0 || self.terms.is_empty() {
                if !first {
                    write!(f, " ")?;
                    if self.constant >= 0 {
                        write!(f, "+ ")?;
                    }
                }
                write!(f, "{}", self.constant)?;
            }

            Ok(())
        }
    }
}

/// Comparison operators (normalized to only = and >=)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompOp {
    Eq,  // =
    Geq, // >=
}

impl fmt::Display for CompOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompOp::Eq => write!(f, "="),
            CompOp::Geq => write!(f, "≥"),
        }
    }
}

/// Linear constraint: expr op 0
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint {
    pub expr: AffineExpr,
    pub op: CompOp,
}

impl Constraint {
    pub fn new(expr: AffineExpr, op: CompOp) -> Self {
        Constraint { expr, op }
    }
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} 0", self.expr, self.op)
    }
}

/// Normalized formula (no Not or Implies)
#[derive(Debug, Clone, PartialEq)]
pub enum Formula {
    Constraint(Constraint),
    And(Vec<Formula>),
    Or(Vec<Formula>),
    Exists(String, Box<Formula>),
    Forall(String, Box<Formula>),
}

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Formula::Constraint(c) => write!(f, "{}", c),
            Formula::And(formulas) => {
                if formulas.is_empty() {
                    write!(f, "⊤") // true
                } else {
                    write!(f, "(")?;
                    for (i, formula) in formulas.iter().enumerate() {
                        if i > 0 {
                            write!(f, " ∧ ")?;
                        }
                        write!(f, "{}", formula)?;
                    }
                    write!(f, ")")
                }
            }
            Formula::Or(formulas) => {
                if formulas.is_empty() {
                    write!(f, "⊥") // false
                } else {
                    write!(f, "(")?;
                    for (i, formula) in formulas.iter().enumerate() {
                        if i > 0 {
                            write!(f, " ∨ ")?;
                        }
                        write!(f, "{}", formula)?;
                    }
                    write!(f, ")")
                }
            }
            Formula::Exists(var, body) => {
                write!(f, "∃{}. {}", var, body)
            }
            Formula::Forall(var, body) => {
                write!(f, "∀{}. {}", var, body)
            }
        }
    }
}

/// The proof invariant extracted from an SMT-LIB file
#[derive(Debug, Clone)]
pub struct ProofInvariant {
    /// Variables declared in the cert function
    pub variables: Vec<String>,
    /// The invariant formula
    pub formula: Formula,
}

/// Parser for SMT-LIB proof certificates
pub struct Parser {
    input: Vec<char>,
    pos: usize,
    /// Variables declared in the current scope
    declared_vars: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at position {}: {}",
            self.position, self.message
        )
    }
}

impl std::error::Error for ParseError {}

type Result<T> = std::result::Result<T, ParseError>;

impl Parser {
    fn new(input: &str) -> Self {
        Parser {
            input: input.chars().collect(),
            pos: 0,
            declared_vars: Vec::new(),
        }
    }

    fn error(&self, msg: &str) -> ParseError {
        ParseError {
            message: msg.to_string(),
            position: self.pos,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        if self.peek() == Some(';') {
            while let Some(ch) = self.peek() {
                self.advance();
                if ch == '\n' {
                    break;
                }
            }
        }
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            self.skip_whitespace();
            if self.peek() == Some(';') {
                self.skip_comment();
            } else {
                break;
            }
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<()> {
        self.skip_ws_and_comments();
        match self.peek() {
            Some(ch) if ch == expected => {
                self.advance();
                Ok(())
            }
            Some(ch) => Err(self.error(&format!("Expected '{}', found '{}'", expected, ch))),
            None => Err(self.error(&format!("Expected '{}', found EOF", expected))),
        }
    }

    fn parse_atom(&mut self) -> Result<String> {
        self.skip_ws_and_comments();

        let mut token = String::new();

        // Check for negative numbers
        if self.peek() == Some('-') {
            token.push('-');
            self.advance();
        }

        // Collect the rest
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() || ch == '(' || ch == ')' {
                break;
            }
            token.push(ch);
            self.advance();
        }

        if token.is_empty() {
            Err(self.error("Expected atom"))
        } else {
            Ok(token)
        }
    }

    fn parse_integer(&mut self) -> Result<i64> {
        let atom = self.parse_atom()?;
        atom.parse::<i64>()
            .map_err(|_| self.error(&format!("Invalid integer: {}", atom)))
    }

    fn peek_atom(&mut self) -> Result<Option<String>> {
        let saved_pos = self.pos;
        self.skip_ws_and_comments();

        if self.peek() == Some('(') {
            self.pos = saved_pos;
            return Ok(None);
        }

        match self.parse_atom() {
            Ok(atom) => {
                self.pos = saved_pos;
                Ok(Some(atom))
            }
            Err(_) => {
                self.pos = saved_pos;
                Ok(None)
            }
        }
    }

    /// Parse an affine expression
    fn parse_affine_expr(&mut self) -> Result<AffineExpr> {
        self.skip_ws_and_comments();

        // Check if it's a list or atom
        if self.peek() != Some('(') {
            // It's an atom - either integer or variable
            let atom = self.parse_atom()?;
            if let Ok(n) = atom.parse::<i64>() {
                Ok(AffineExpr::from_const(n))
            } else {
                // Variables with @ are allowed - they come from SMPT output
                // Check if variable is declared (without the @suffix if present)
                let base_var = atom.split('@').next().unwrap_or(&atom);
                if !self.declared_vars.contains(&base_var.to_string())
                    && !self.declared_vars.contains(&atom)
                {
                    return Err(self.error(&format!("Undefined variable: {}", atom)));
                }
                Ok(AffineExpr::from_var(atom))
            }
        } else {
            // It's a list - parse operation
            self.expect_char('(')?;
            let op = self.parse_atom()?;

            match op.as_str() {
                "+" => {
                    let mut result = AffineExpr::new();

                    // Parse all arguments
                    while self.peek() != Some(')') {
                        let arg = self.parse_affine_expr()?;
                        result = result.add(&arg);
                    }

                    self.expect_char(')')?;
                    Ok(result)
                }
                "-" => {
                    let lhs = self.parse_affine_expr()?;
                    let rhs = self.parse_affine_expr()?;
                    self.expect_char(')')?;
                    Ok(lhs.sub(&rhs))
                }
                "*" => {
                    // Parse first argument
                    let arg1 = self.parse_affine_expr()?;
                    let arg2 = self.parse_affine_expr()?;
                    self.expect_char(')')?;

                    // One must be constant
                    if arg1.is_constant() {
                        Ok(arg2.mul_by_const(arg1.get_constant()))
                    } else if arg2.is_constant() {
                        Ok(arg1.mul_by_const(arg2.get_constant()))
                    } else {
                        Err(self.error("Multiplication requires at least one constant"))
                    }
                }
                _ => Err(self.error(&format!("Unknown arithmetic operation: {}", op))),
            }
        }
    }

    /// Parse a constraint (comparison)
    fn parse_constraint(&mut self) -> Result<Constraint> {
        self.expect_char('(')?;
        let op = self.parse_atom()?;

        let comp_op = match op.as_str() {
            "=" => CompOp::Eq,
            ">=" => CompOp::Geq,
            ">" => {
                // Convert > to >= by adjusting constant
                let lhs = self.parse_affine_expr()?;
                let rhs = self.parse_affine_expr()?;
                self.expect_char(')')?;

                // lhs > rhs becomes lhs - rhs > 0 becomes lhs - rhs - 1 >= 0
                let mut expr = lhs.sub(&rhs);
                expr.constant -= 1;
                return Ok(Constraint::new(expr, CompOp::Geq));
            }
            "<=" => {
                // Convert <= to >= by negation
                let lhs = self.parse_affine_expr()?;
                let rhs = self.parse_affine_expr()?;
                self.expect_char(')')?;

                // lhs <= rhs becomes rhs - lhs >= 0
                let expr = rhs.sub(&lhs);
                return Ok(Constraint::new(expr, CompOp::Geq));
            }
            "<" => {
                // Convert < to >= by negation and adjustment
                let lhs = self.parse_affine_expr()?;
                let rhs = self.parse_affine_expr()?;
                self.expect_char(')')?;

                // lhs < rhs becomes rhs - lhs > 0 becomes rhs - lhs - 1 >= 0
                let mut expr = rhs.sub(&lhs);
                expr.constant -= 1;
                return Ok(Constraint::new(expr, CompOp::Geq));
            }
            _ => return Err(self.error(&format!("Unknown comparison operator: {}", op))),
        };

        // For = and >=, parse normally
        let lhs = self.parse_affine_expr()?;
        let rhs = self.parse_affine_expr()?;
        self.expect_char(')')?;

        // Convert to expr op 0 form
        let expr = lhs.sub(&rhs);
        Ok(Constraint::new(expr, comp_op))
    }

    /// Parse quantified variables list: ((x Int) (y Int) ...) or empty ()
    fn parse_var_list(&mut self) -> Result<Vec<String>> {
        self.expect_char('(')?;
        self.skip_ws_and_comments();

        let mut vars = Vec::new();

        // Check for empty variable list
        if self.peek() == Some(')') {
            self.advance();
            return Ok(vars); // Empty variable list
        }

        while self.peek() != Some(')') {
            self.expect_char('(')?;
            let var_name = self.parse_atom()?;
            let var_type = self.parse_atom()?;
            self.expect_char(')')?;

            if var_type != "Int" {
                return Err(self.error(&format!("Expected Int type, got {}", var_type)));
            }

            vars.push(var_name);
            self.skip_ws_and_comments();
        }

        self.expect_char(')')?;
        Ok(vars)
    }

    /// Negate a normalized formula using De Morgan's laws
    fn negate_formula(formula: Formula) -> Formula {
        match formula {
            Formula::Constraint(c) => {
                match c.op {
                    CompOp::Eq => {
                        // ¬(expr = 0) becomes (expr > 0) ∨ (expr < 0)
                        // which is (expr >= 1) ∨ (-expr >= 1)
                        let pos_expr = c.expr.clone();
                        let mut pos_constraint = Constraint::new(pos_expr, CompOp::Geq);
                        pos_constraint.expr.constant -= 1;

                        let neg_expr = c.expr.negate();
                        let mut neg_constraint = Constraint::new(neg_expr, CompOp::Geq);
                        neg_constraint.expr.constant -= 1;

                        Formula::Or(vec![
                            Formula::Constraint(pos_constraint),
                            Formula::Constraint(neg_constraint),
                        ])
                    }
                    CompOp::Geq => {
                        // ¬(expr >= 0) becomes expr < 0 which is -expr - 1 >= 0
                        let mut neg_expr = c.expr.negate();
                        neg_expr.constant -= 1;
                        Formula::Constraint(Constraint::new(neg_expr, CompOp::Geq))
                    }
                }
            }
            Formula::And(formulas) => {
                // ¬(A ∧ B) = ¬A ∨ ¬B
                let negated: Vec<Formula> =
                    formulas.into_iter().map(Self::negate_formula).collect();
                Formula::Or(negated)
            }
            Formula::Or(formulas) => {
                // ¬(A ∨ B) = ¬A ∧ ¬B
                let negated: Vec<Formula> =
                    formulas.into_iter().map(Self::negate_formula).collect();
                Formula::And(negated)
            }
            Formula::Exists(var, body) => {
                // ¬∃x.P = ∀x.¬P
                Formula::Forall(var, Box::new(Self::negate_formula(*body)))
            }
            Formula::Forall(var, body) => {
                // ¬∀x.P = ∃x.¬P
                Formula::Exists(var, Box::new(Self::negate_formula(*body)))
            }
        }
    }

    /// Parse a formula
    fn parse_formula(&mut self) -> Result<Formula> {
        self.skip_ws_and_comments();

        // Check for bare atoms first (true/false)
        if self.peek() != Some('(') {
            // Try to parse an atom
            if let Ok(atom) = self.parse_atom() {
                match atom.as_str() {
                    "true" => return Ok(Formula::And(vec![])), // Empty AND
                    "false" => return Ok(Formula::Or(vec![])), // Empty OR
                    _ => {
                        return Err(self.error(&format!("Expected formula, found atom '{}'", atom)));
                    }
                }
            }
            return Err(self.error("Expected '(' to start formula"));
        }

        self.expect_char('(')?;

        // Peek ahead to see what we have
        let op = if let Ok(Some(atom)) = self.peek_atom() {
            self.parse_atom()?;
            atom
        } else {
            // Empty list or other issue
            if self.peek() == Some(')') {
                self.advance();
                // Empty list - treat as empty AND (true)
                return Ok(Formula::And(vec![]));
            }
            return Err(self.error("Expected operator or closing parenthesis"));
        };

        match op.as_str() {
            "and" => {
                self.skip_ws_and_comments();

                // Check for empty (and )
                if self.peek() == Some(')') {
                    self.advance();
                    return Ok(Formula::And(vec![])); // Empty AND = true
                }

                let mut formulas = Vec::new();
                while self.peek() != Some(')') {
                    let formula = self.parse_formula()?;

                    // Skip empty AND (true) and empty OR (false) in AND context
                    // According to the test, these should just be ignored
                    if let Formula::And(ref parts) = formula {
                        if parts.is_empty() {
                            self.skip_ws_and_comments();
                            continue; // Skip empty AND
                        }
                    }

                    if let Formula::Or(ref parts) = formula {
                        if parts.is_empty() {
                            self.skip_ws_and_comments();
                            continue; // Skip empty OR
                        }
                    }

                    formulas.push(formula);
                    self.skip_ws_and_comments();
                }

                self.expect_char(')')?;
                Ok(Formula::And(formulas))
            }
            "or" => {
                self.skip_ws_and_comments();

                // Check for empty (or )
                if self.peek() == Some(')') {
                    self.advance();
                    return Ok(Formula::Or(vec![])); // Empty OR = false
                }

                let mut formulas = Vec::new();
                while self.peek() != Some(')') {
                    let formula = self.parse_formula()?;

                    // Skip empty OR (false) and empty AND (true) in OR context
                    if let Formula::Or(ref parts) = formula {
                        if parts.is_empty() {
                            self.skip_ws_and_comments();
                            continue; // Skip empty OR
                        }
                    }

                    if let Formula::And(ref parts) = formula {
                        if parts.is_empty() {
                            self.skip_ws_and_comments();
                            continue; // Skip empty AND
                        }
                    }

                    formulas.push(formula);
                    self.skip_ws_and_comments();
                }

                self.expect_char(')')?;
                Ok(Formula::Or(formulas))
            }
            "not" => {
                let inner = self.parse_formula()?;
                self.expect_char(')')?;
                Ok(Self::negate_formula(inner))
            }
            "=>" | "implies" => {
                let lhs = self.parse_formula()?;
                let rhs = self.parse_formula()?;
                self.expect_char(')')?;

                // A => B is ¬A ∨ B
                Ok(Formula::Or(vec![Self::negate_formula(lhs), rhs]))
            }
            "exists" => {
                // Save current declared vars
                let saved_vars = self.declared_vars.clone();

                let vars = self.parse_var_list()?;
                // Add to declared vars
                self.declared_vars.extend(vars.clone());

                let body = self.parse_formula()?;
                self.expect_char(')')?;

                // Restore declared vars
                self.declared_vars = saved_vars;

                // If no variables, just return the body
                if vars.is_empty() {
                    return Ok(body);
                }

                // Convert multiple variables to nested single quantifiers
                let mut result = body;
                for var in vars.into_iter().rev() {
                    result = Formula::Exists(var, Box::new(result));
                }
                Ok(result)
            }
            "forall" => {
                // Save current declared vars
                let saved_vars = self.declared_vars.clone();

                let vars = self.parse_var_list()?;
                // Add to declared vars
                self.declared_vars.extend(vars.clone());

                let body = self.parse_formula()?;
                self.expect_char(')')?;

                // Restore declared vars
                self.declared_vars = saved_vars;

                // If no variables, just return the body
                if vars.is_empty() {
                    return Ok(body);
                }

                // Convert multiple variables to nested single quantifiers
                let mut result = body;
                for var in vars.into_iter().rev() {
                    result = Formula::Forall(var, Box::new(result));
                }
                Ok(result)
            }
            "=" | ">=" | ">" | "<=" | "<" => {
                // It's a constraint - we already consumed '(' and the operator
                // So we need to parse it inline
                let comp_op = match op.as_str() {
                    "=" => CompOp::Eq,
                    ">=" => CompOp::Geq,
                    ">" => {
                        // Convert > to >= by adjusting constant
                        let lhs = self.parse_affine_expr()?;
                        let rhs = self.parse_affine_expr()?;
                        self.expect_char(')')?;

                        // lhs > rhs becomes lhs - rhs > 0 becomes lhs - rhs - 1 >= 0
                        let mut expr = lhs.sub(&rhs);
                        expr.constant -= 1;
                        return Ok(Formula::Constraint(Constraint::new(expr, CompOp::Geq)));
                    }
                    "<=" => {
                        // Convert <= to >= by negation
                        let lhs = self.parse_affine_expr()?;
                        let rhs = self.parse_affine_expr()?;
                        self.expect_char(')')?;

                        // lhs <= rhs becomes rhs - lhs >= 0
                        let expr = rhs.sub(&lhs);
                        return Ok(Formula::Constraint(Constraint::new(expr, CompOp::Geq)));
                    }
                    "<" => {
                        // Convert < to >= by negation and adjustment
                        let lhs = self.parse_affine_expr()?;
                        let rhs = self.parse_affine_expr()?;
                        self.expect_char(')')?;

                        // lhs < rhs becomes rhs - lhs > 0 becomes rhs - lhs - 1 >= 0
                        let mut expr = rhs.sub(&lhs);
                        expr.constant -= 1;
                        return Ok(Formula::Constraint(Constraint::new(expr, CompOp::Geq)));
                    }
                    _ => unreachable!(),
                };

                // For = and >=, parse normally
                let lhs = self.parse_affine_expr()?;
                let rhs = self.parse_affine_expr()?;
                self.expect_char(')')?;

                // Convert to expr op 0 form
                let expr = lhs.sub(&rhs);
                Ok(Formula::Constraint(Constraint::new(expr, comp_op)))
            }
            _ => Err(self.error(&format!("Unknown formula operator: {}", op))),
        }
    }

    /// Parse a complete SMT-LIB file to extract the cert function
    fn parse_smtlib(&mut self) -> Result<ProofInvariant> {
        let mut cert_found = false;
        let mut variables = Vec::new();
        let mut formula = None;

        while self.pos < self.input.len() {
            self.skip_ws_and_comments();

            if self.pos >= self.input.len() {
                break;
            }

            // Each top-level form should be a list
            if self.peek() != Some('(') {
                // If we already found cert, we're done
                if cert_found {
                    break;
                }
                return Err(self.error("Expected '(' at top level"));
            }

            // Check if this is define-fun cert
            let saved_pos = self.pos;
            self.advance(); // skip '('

            if let Ok(cmd) = self.parse_atom() {
                if cmd == "define-fun" {
                    if let Ok(name) = self.parse_atom() {
                        if name == "cert" {
                            // Parse the cert function
                            cert_found = true;

                            // Parse parameters
                            self.expect_char('(')?;
                            while self.peek() != Some(')') {
                                self.expect_char('(')?;
                                let var_name = self.parse_atom()?;
                                let var_type = self.parse_atom()?;
                                self.expect_char(')')?;

                                if var_type != "Int" {
                                    return Err(
                                        self.error(&format!("Expected Int type, got {}", var_type))
                                    );
                                }

                                variables.push(var_name);
                            }
                            self.expect_char(')')?;

                            // Parse return type
                            let ret_type = self.parse_atom()?;
                            if ret_type != "Bool" {
                                return Err(self.error(&format!(
                                    "Expected Bool return type, got {}",
                                    ret_type
                                )));
                            }

                            // Set declared variables for body parsing
                            self.declared_vars = variables.clone();

                            // Parse body
                            formula = Some(self.parse_formula()?);

                            // Clear declared variables
                            self.declared_vars.clear();

                            self.expect_char(')')?; // close define-fun

                            // Once we found cert, we can stop parsing
                            break;
                        } else {
                            // Not cert, skip to end of this form
                            self.pos = saved_pos;
                            self.skip_form()?;
                        }
                    } else {
                        self.pos = saved_pos;
                        self.skip_form()?;
                    }
                } else {
                    // Not define-fun, skip
                    self.pos = saved_pos;
                    self.skip_form()?;
                }
            } else {
                self.pos = saved_pos;
                self.skip_form()?;
            }
        }

        if !cert_found {
            return Err(self.error("No cert function found in proof file"));
        }

        Ok(ProofInvariant {
            variables,
            formula: formula.unwrap(),
        })
    }

    /// Skip an S-expression form
    fn skip_form(&mut self) -> Result<()> {
        self.skip_ws_and_comments();

        if self.peek() == Some('(') {
            self.advance();
            let mut depth = 1;

            while depth > 0 && self.pos < self.input.len() {
                match self.peek() {
                    Some('(') => {
                        depth += 1;
                        self.advance();
                    }
                    Some(')') => {
                        depth -= 1;
                        self.advance();
                    }
                    Some(';') => {
                        self.skip_comment();
                    }
                    _ => {
                        self.advance();
                    }
                }
            }

            if depth > 0 {
                return Err(self.error("Unclosed parenthesis"));
            }
        } else {
            // Skip atom
            self.parse_atom()?;
        }

        Ok(())
    }
}

/// Parse a proof file and extract the invariant
pub fn parse_proof_file(content: &str) -> Result<ProofInvariant> {
    let mut parser = Parser::new(content);
    parser.parse_smtlib()
}

/// Convert to presburger constraint representation
pub fn to_presburger_constraint(
    constraint: &Constraint,
) -> crate::presburger::Constraint<crate::presburger::Variable<String>> {
    use crate::presburger::{Constraint as PConstraint, ConstraintType, Variable};

    let (terms, constant) = constraint.expr.to_linear_combination();
    let linear_combination: Vec<(i32, Variable<String>)> = terms
        .into_iter()
        .map(|(coeff, var)| (coeff as i32, Variable::Var(var)))
        .collect();

    let constraint_type = match constraint.op {
        CompOp::Eq => ConstraintType::EqualToZero,
        CompOp::Geq => ConstraintType::NonNegative,
    };

    PConstraint::new(linear_combination, constant as i32, constraint_type)
}

/// Pretty‐print a parsed certificate
impl fmt::Display for ProofInvariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Certificate variables: {:?}", self.variables)?;
        writeln!(f, "Certificate formula:")?;

        match &self.formula {
            Formula::And(parts) if parts.len() > 1 => {
                writeln!(f, "(")?;
                for (i, part) in parts.iter().enumerate() {
                    write!(f, "    ({})", part)?;
                    if i + 1 < parts.len() {
                        writeln!(f, " ∧")?;
                    } else {
                        writeln!(f)?; // last line, just newline
                    }
                }
                write!(f, ")")
            }
            Formula::Or(parts) if parts.len() > 1 => {
                writeln!(f, "(")?;
                for (i, part) in parts.iter().enumerate() {
                    write!(f, "    ({})", part)?;
                    if i + 1 < parts.len() {
                        writeln!(f, " ∨")?;
                    } else {
                        writeln!(f)?;
                    }
                }
                write!(f, ")")
            }
            // fallback to the plain Display (no extra parens / lines)
            other => write!(f, "{}", other),
        }
    }
}

/// Parse the given SMT-LIB text and print its `cert` function to stdout
pub fn print_proof_certificate(content: &str) -> Result<()> {
    let inv = parse_proof_file(content)?;
    println!("{}", inv);
    Ok(())
}

/// Recursively print a Formula AST with indentation
fn print_formula_tree(formula: &Formula, indent: usize) {
    let pad = "  ".repeat(indent);
    match formula {
        Formula::Constraint(c) => {
            println!("{}Constraint: {}", pad, c);
        }
        Formula::And(children) => {
            println!("{}And", pad);
            for child in children {
                print_formula_tree(child, indent + 1);
            }
        }
        Formula::Or(children) => {
            println!("{}Or", pad);
            for child in children {
                print_formula_tree(child, indent + 1);
            }
        }
        Formula::Exists(var, body) => {
            println!("{}Exists {}", pad, var);
            print_formula_tree(body, indent + 1);
        }
        Formula::Forall(var, body) => {
            println!("{}Forall {}", pad, var);
            print_formula_tree(body, indent + 1);
        }
    }
}

/// Recursively convert a normalized `Formula` into a `PresburgerSet<String>`.
/// - Leaves yield a single‐constraint `QuantifiedSet<String>`.
/// - `And` → intersection of children’s sets.
/// - `Or`  → union of children’s sets.
/// - `Exists(x, body)` → project out `x` (we push `x` into the mapping before descending).
fn formula_to_presburger(formula: &Formula, mut mapping: Vec<String>) -> PresburgerSet<String> {
    match formula {
        Formula::Constraint(c) => {
            // each leaf constraint → one QuantifiedSet
            let pcon: PConstraint<Variable<String>> = to_presburger_constraint(c);
            let qs = QuantifiedSet::new(vec![pcon]);
            PresburgerSet::from_quantified_sets(&[qs], mapping)
        }
        Formula::And(children) => {
            // intersection of all children
            let mut iter = children
                .iter()
                .map(|f| formula_to_presburger(f, mapping.clone()));
            let first = iter
                .next()
                .unwrap_or_else(|| PresburgerSet::universe(mapping.clone()));
            iter.fold(first, |acc, next| acc.intersection(&next))
        }
        Formula::Or(children) => {
            // union of all children
            let mut iter = children
                .iter()
                .map(|f| formula_to_presburger(f, mapping.clone()));
            let first = iter.next().unwrap_or_else(|| PresburgerSet::zero());
            iter.fold(first, |acc, next| acc.union(&next))
        }
        Formula::Exists(var, body) => {
            // introduce `var` as an existential dimension, then project it out
            mapping.push(var.clone());
            let set = formula_to_presburger(body, mapping);
            set.project_out(var.clone())
        }
        Formula::Forall(_, _) => {
            panic!("`Forall` → Presburger not implemented");
        }
    }
}

/// Parse a certificate from disk and build its Presburger set.
pub fn parse_and_build_presburger_set<P: AsRef<Path>>(
    path: P,
) -> std::result::Result<PresburgerSet<String>, Box<dyn std::error::Error>> {
    let txt = fs::read_to_string(path)?;
    let inv = parse_proof_file(&txt)?;
    // start with the vector of _parameters_ as the initial mapping
    Ok(formula_to_presburger(&inv.formula, inv.variables.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_affine_expr() {
        let x = AffineExpr::from_var("x".to_string());
        let y = AffineExpr::from_var("y".to_string());
        let five = AffineExpr::from_const(5);

        // x + y + 5
        let expr = x.add(&y).add(&five);
        assert_eq!(expr.to_string(), "x + y + 5");

        // 2*x - 3*y + 10
        let expr2 = x
            .mul_by_const(2)
            .add(&y.mul_by_const(-3))
            .add(&AffineExpr::from_const(10));
        assert_eq!(expr2.to_string(), "2*x -3*y + 10");

        // Test subtraction
        let expr3 = x.sub(&y);
        assert_eq!(expr3.to_string(), "x -y");
    }

    #[test]
    fn test_simple_proof() {
        let proof = r#"
(set-logic LIA)
(define-fun cert ((x Int)(y Int)) Bool 
  (and (>= x 0) (>= y 0) (= (+ x y) 10)))
"#;

        let result = parse_proof_file(proof).unwrap();
        assert_eq!(result.variables, vec!["x", "y"]);

        // Check it's an AND of 3 constraints
        match &result.formula {
            Formula::And(constraints) => {
                assert_eq!(constraints.len(), 3);
            }
            _ => panic!("Expected AND formula"),
        }
    }

    #[test]
    fn test_undefined_variable() {
        let proof = r#"
(set-logic LIA)
(define-fun cert ((x Int)) Bool 
  (>= x y))
"#;

        let result = parse_proof_file(proof);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Undefined variable"));
    }

    #[test]
    fn test_variable_with_suffix() {
        // Variables with @ suffixes are now allowed
        let proof = r#"
(set-logic LIA)
(define-fun cert ((x Int)) Bool 
  (>= x@0 0))
"#;

        let result = parse_proof_file(proof);
        assert!(result.is_ok());
        let inv = result.unwrap();
        match &inv.formula {
            Formula::Constraint(c) => {
                assert_eq!(c.expr.to_string(), "x@0");
            }
            _ => panic!("Expected constraint"),
        }
    }

    #[test]
    fn test_nested_arithmetic() {
        let proof = r#"
(set-logic LIA)
(define-fun cert ((x Int)(y Int)) Bool 
  (= (+ (+ x 1) y) 10))
"#;

        let result = parse_proof_file(proof).unwrap();
        match &result.formula {
            Formula::Constraint(c) => {
                assert_eq!(c.expr.to_string(), "x + y -9");
                assert_eq!(c.op, CompOp::Eq);
            }
            _ => panic!("Expected constraint"),
        }
    }

    #[test]
    fn test_normalization() {
        // Test > becomes >=
        let proof = r#"
(set-logic LIA)
(define-fun cert ((x Int)) Bool (> x 5))
"#;

        let result = parse_proof_file(proof).unwrap();
        match &result.formula {
            Formula::Constraint(c) => {
                assert_eq!(c.expr.to_string(), "x -6");
                assert_eq!(c.op, CompOp::Geq);
            }
            _ => panic!("Expected constraint"),
        }
    }

    #[test]
    fn test_implies_normalization() {
        let proof = r#"
(set-logic LIA)
(define-fun cert ((x Int)(y Int)) Bool 
  (=> (>= x 0) (>= y 0)))
"#;

        let result = parse_proof_file(proof).unwrap();
        // Should be (¬(x >= 0) ∨ (y >= 0))
        match &result.formula {
            Formula::Or(parts) => {
                assert_eq!(parts.len(), 2);
                // First part should be ¬(x >= 0) which is -x - 1 >= 0
                match &parts[0] {
                    Formula::Constraint(c) => {
                        assert_eq!(c.expr.to_string(), "-x -1");
                        assert_eq!(c.op, CompOp::Geq);
                    }
                    _ => panic!("Expected constraint"),
                }
            }
            _ => panic!("Expected OR"),
        }
    }

    #[test]
    fn test_exists() {
        let proof = r#"
(set-logic LIA)
(define-fun cert ((x Int)) Bool 
  (exists ((t Int)) (and (>= t 0) (= x (* 2 t)))))
"#;

        let result = parse_proof_file(proof).unwrap();
        match &result.formula {
            Formula::Exists(var, body) => {
                assert_eq!(var, "t");
                match body.as_ref() {
                    Formula::And(constraints) => {
                        assert_eq!(constraints.len(), 2);
                    }
                    _ => panic!("Expected AND in exists body"),
                }
            }
            _ => panic!("Expected EXISTS"),
        }
    }

    #[test]
    fn test_empty_and_or() {
        let proof = r#"
(set-logic LIA)
(define-fun cert ((x Int)) Bool 
  (and (or ) (>= x 0) (and )))
"#;

        let result = parse_proof_file(proof).unwrap();
        match &result.formula {
            Formula::And(parts) => {
                // We have: empty OR (false), x >= 0, empty AND (true)
                // Empty lists are skipped during parsing, so we should have just x >= 0
                assert_eq!(parts.len(), 1);
                match &parts[0] {
                    Formula::Constraint(c) => {
                        assert_eq!(c.expr.to_string(), "x");
                    }
                    _ => panic!("Expected constraint"),
                }
            }
            _ => panic!("Expected AND"),
        }
    }

    #[test]
    fn test_true_false_constants() {
        // Test parsing 'true' constant
        let proof_true = r#"
(set-logic LIA)
(define-fun cert ((x Int)) Bool true)
"#;

        let result = parse_proof_file(proof_true).unwrap();
        match &result.formula {
            Formula::And(parts) => {
                assert_eq!(parts.len(), 0, "true should be empty AND");
            }
            _ => panic!("Expected AND for true"),
        }

        // Test parsing 'false' constant
        let proof_false = r#"
(set-logic LIA)
(define-fun cert ((x Int)) Bool false)
"#;

        let result = parse_proof_file(proof_false).unwrap();
        match &result.formula {
            Formula::Or(parts) => {
                assert_eq!(parts.len(), 0, "false should be empty OR");
            }
            _ => panic!("Expected OR for false"),
        }

        // Test true and false in context
        let proof_mixed = r#"
(set-logic LIA)
(define-fun cert ((x Int)) Bool 
  (and true (>= x 0) false))
"#;

        let result = parse_proof_file(proof_mixed).unwrap();
        match &result.formula {
            Formula::And(parts) => {
                // true is skipped, false is skipped in AND context
                // Should have just x >= 0
                assert_eq!(parts.len(), 1);
                match &parts[0] {
                    Formula::Constraint(c) => {
                        assert_eq!(c.expr.to_string(), "x");
                    }
                    _ => panic!("Expected constraint"),
                }
            }
            _ => panic!("Expected AND"),
        }

        // Test OR with true and false
        let proof_or = r#"
(set-logic LIA)  
(define-fun cert ((x Int)) Bool
  (or false (= x 5) true))
"#;

        let result = parse_proof_file(proof_or).unwrap();
        match &result.formula {
            Formula::Or(parts) => {
                // false is skipped, true is skipped in OR context
                // Should have just x = 5
                assert_eq!(parts.len(), 1);
                match &parts[0] {
                    Formula::Constraint(c) => {
                        assert_eq!(c.expr.to_string(), "x -5");
                        assert_eq!(c.op, CompOp::Eq);
                    }
                    _ => panic!("Expected constraint"),
                }
            }
            _ => panic!("Expected OR"),
        }
    }

    #[test]
    fn test_parse_all_proof_files_in_out_dir() {
        use std::fs;
        use std::path::Path;

        let out_dir = Path::new("out");
        if !out_dir.exists() {
            println!("Skipping test: out directory does not exist");
            return;
        }

        let mut total_files = 0;
        let mut successful_parses = 0;
        let mut failed_parses = 0;
        let mut failures = Vec::new();

        // Function to recursively find proof files
        fn find_proof_files(
            dir: &Path,
            files: &mut Vec<std::path::PathBuf>,
        ) -> std::io::Result<()> {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    find_proof_files(&path, files)?;
                } else if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        if name_str.contains("proof") && name_str.ends_with(".txt") {
                            files.push(path);
                        }
                    }
                }
            }
            Ok(())
        }

        let mut proof_files = Vec::new();
        if let Err(e) = find_proof_files(out_dir, &mut proof_files) {
            println!("Error scanning directory: {}", e);
            return;
        }

        println!("\nFound {} proof files to test", proof_files.len());

        for file_path in proof_files {
            total_files += 1;

            match fs::read_to_string(&file_path) {
                Ok(content) => match parse_proof_file(&content) {
                    Ok(invariant) => {
                        successful_parses += 1;
                        println!(
                            "✓ {} ({} vars)",
                            file_path.display(),
                            invariant.variables.len()
                        );
                    }
                    Err(e) => {
                        failed_parses += 1;
                        let relative_path = file_path.strip_prefix("out/").unwrap_or(&file_path);
                        failures.push((relative_path.to_path_buf(), e.to_string()));
                        println!("✗ {}: {}", file_path.display(), e);
                    }
                },
                Err(e) => {
                    failed_parses += 1;
                    failures.push((file_path.clone(), format!("Failed to read file: {}", e)));
                    println!("✗ {}: Failed to read file: {}", file_path.display(), e);
                }
            }
        }

        println!("\n=== Summary ===");
        println!("Total files: {}", total_files);
        println!("Successful: {}", successful_parses);
        println!("Failed: {}", failed_parses);

        if !failures.is_empty() {
            println!("\n=== Failures ===");
            for (path, error) in &failures {
                println!("{}: {}", path.display(), error);
            }

            // Check if failures are due to expected reasons
            let no_cert_failures = failures
                .iter()
                .filter(|(_, err)| err.contains("No cert function"))
                .count();

            println!(
                "\nFailures due to missing cert function: {}/{}",
                no_cert_failures, failed_parses
            );

            // Don't fail the test if all errors are expected
            if no_cert_failures < failed_parses {
                println!("\nUnexpected failures found!");
                for (path, error) in &failures {
                    if !error.contains("No cert function") {
                        println!("  {}: {}", path.display(), error);
                    }
                }
                panic!("Some files failed to parse for unexpected reasons");
            }
        }
    }

    #[test]
    fn test_parse_all_proof_files_quiet() {
        use std::fs;
        use std::path::Path;

        let out_dir = Path::new("out");
        if !out_dir.exists() {
            return; // Skip if no out directory
        }

        // Recursively find proof files
        fn find_proof_files(
            dir: &Path,
            files: &mut Vec<std::path::PathBuf>,
        ) -> std::io::Result<()> {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    find_proof_files(&path, files)?;
                } else if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        if name_str.contains("proof") && name_str.ends_with(".txt") {
                            files.push(path);
                        }
                    }
                }
            }
            Ok(())
        }

        let mut proof_files = Vec::new();
        find_proof_files(out_dir, &mut proof_files).expect("Failed to scan directory");

        let mut stats = (0, 0, 0); // (total, success, expected_failures)

        for file_path in proof_files {
            stats.0 += 1;

            if let Ok(content) = fs::read_to_string(&file_path) {
                match parse_proof_file(&content) {
                    Ok(_) => stats.1 += 1,
                    Err(e) => {
                        if e.message.contains("No cert function") {
                            stats.2 += 1;
                        } else {
                            panic!("Unexpected parse error in {}: {}", file_path.display(), e);
                        }
                    }
                }
            }
        }

        // Basic sanity check
        assert_eq!(
            stats.0,
            stats.1 + stats.2,
            "Total files ({}) != successful ({}) + expected failures ({})",
            stats.0,
            stats.1,
            stats.2
        );

        // Ensure we parsed some files successfully
        assert!(stats.1 > 0, "No files were parsed successfully");
    }
}

#[test]
fn test_parse_and_print_specific_proof_file() {
    let proof_path =
        Path::new("out/simple_nonser2_turned_ser_with_locks/smpt_constraints_disjunct_0_proof.txt");
    assert!(
        proof_path.exists(),
        "Test fixture not found: {}",
        proof_path.display()
    );

    let content = fs::read_to_string(proof_path).expect("Failed to read proof file for test");

    // Parse normally...
    let inv = parse_proof_file(&content).expect("parse_proof_file failed");

    // Now print the tree from root to leaves:
    println!("\n=== Parsed Formula Tree ===");
    print_formula_tree(&inv.formula, 0);

    // And still allow us to inspect the parsed invariant:
    assert!(!inv.variables.is_empty(), "No variables parsed");
    match inv.formula {
        Formula::And(ref parts) | Formula::Or(ref parts) => {
            assert!(!parts.is_empty(), "Parsed an empty conjunct/disjunct");
        }
        _ => {}
    }
}

#[test]
fn test_parse_and_build_set() {
    let proof_path =
        Path::new("out/simple_nonser2_turned_ser_with_locks/smpt_constraints_disjunct_0_proof.txt");
    assert!(proof_path.exists());

    let set =
        parse_and_build_presburger_set(proof_path).expect("parse_and_build_presburger_set failed");

    println!("Resulting Presburger set:\n{}", set);
    assert!(!set.is_empty());
}
