use crate::ns::*;
use crate::parser::*;
use hash_cons::Hc;

use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
#[derive(Clone, Eq, PartialEq)]
pub struct Env {
    vars: HashMap<String, i64>,
}

impl std::fmt::Display for Env {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Sort variables for consistent output
        let mut pairs: Vec<_> = self.vars.iter().collect();
        pairs.sort_by_key(|(k, _)| *k);

        // Format each variable assignment
        let formatted = pairs
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(",");

        write!(f, "{{{}}}", formatted)
    }
}

impl Hash for Env {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Convert to a sorted list of (key, value) pairs and hash that
        let mut pairs = self.vars.iter().collect::<Vec<_>>();
        pairs.sort_by_key(|(key, val)| (*key, *val));
        for (key, value) in pairs {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl Env {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }
    fn insert(self, var: String, value: i64) -> Self {
        let mut vars = self.vars.clone();
        vars.insert(var, value);
        Self { vars }
    }
    fn get(&self, var: &str) -> i64 {
        // Variables are initialized to 0
        *self.vars.get(var).unwrap_or(&0)
    }
}

pub type Local = Env;
pub type Global = Env;
pub enum ExprResult {
    Yielding(Hc<Expr>),
    Returning(i64),
}

fn is_local(var: &str) -> bool {
    // Variables that start with a lowercase letter are local
    var.chars().next().unwrap().is_lowercase()
}

pub fn run_expr(
    exprhc: &mut ExprHc,
    expr: &Expr,
    local: Local,
    global: Global,
) -> Vec<(ExprResult, Local, Global)> {
    let mut results = Vec::new();
    match expr {
        Expr::Assign(var, e) => {
            for (expr_result, local, global) in run_expr(exprhc, e, local, global) {
                match expr_result {
                    ExprResult::Yielding(e) => {
                        results.push((
                            ExprResult::Yielding(exprhc.assign(var.clone(), e)),
                            local,
                            global,
                        ));
                    }
                    ExprResult::Returning(n) => {
                        // Assign to local or global
                        if is_local(var) {
                            results.push((
                                ExprResult::Returning(n),
                                local.insert(var.clone(), n),
                                global,
                            ));
                        } else {
                            results.push((
                                ExprResult::Returning(n),
                                local,
                                global.insert(var.clone(), n),
                            ));
                        }
                    }
                }
            }
        }
        Expr::Equal(e1, e2) => {
            for (expr_result1, local1, global1) in run_expr(exprhc, e1, local, global) {
                match expr_result1 {
                    ExprResult::Yielding(e) => {
                        results.push((
                            ExprResult::Yielding(exprhc.equal(e, e2.clone())),
                            local1,
                            global1,
                        ));
                    }
                    ExprResult::Returning(n1) => {
                        for (expr_result2, local2, global2) in run_expr(exprhc, e2, local1, global1)
                        {
                            match expr_result2 {
                                ExprResult::Yielding(e) => {
                                    let e1 = exprhc.number(n1);
                                    let e = exprhc.equal(e1, e);
                                    results.push((ExprResult::Yielding(e), local2, global2));
                                }
                                ExprResult::Returning(n2) => {
                                    let result = if n1 == n2 { 1 } else { 0 };
                                    results.push((ExprResult::Returning(result), local2, global2));
                                }
                            }
                        }
                    }
                }
            }
        }
        Expr::Sequence(e1, e2) => {
            for (expr_result1, local1, global1) in run_expr(exprhc, e1, local, global) {
                match expr_result1 {
                    ExprResult::Yielding(e) => {
                        results.push((
                            ExprResult::Yielding(exprhc.sequence(e, e2.clone())),
                            local1,
                            global1,
                        ));
                    }
                    ExprResult::Returning(_) => {
                        // Ignore the result of e1 and continue with e2
                        for (expr_result2, local2, global2) in run_expr(exprhc, e2, local1, global1)
                        {
                            results.push((expr_result2, local2, global2));
                        }
                    }
                }
            }
        }
        Expr::If(cond, then_branch, else_branch) => {
            for (expr_result, local1, global1) in run_expr(exprhc, cond, local, global) {
                match expr_result {
                    ExprResult::Yielding(e) => {
                        results.push((
                            ExprResult::Yielding(exprhc.if_expr(
                                e,
                                then_branch.clone(),
                                else_branch.clone(),
                            )),
                            local1,
                            global1,
                        ));
                    }
                    ExprResult::Returning(n) => {
                        if n != 0 {
                            // Condition is true, execute then branch
                            for (expr_result2, local2, global2) in
                                run_expr(exprhc, then_branch, local1, global1)
                            {
                                results.push((expr_result2, local2, global2));
                            }
                        } else {
                            // Condition is false, execute else branch
                            for (expr_result2, local2, global2) in
                                run_expr(exprhc, else_branch, local1, global1)
                            {
                                results.push((expr_result2, local2, global2));
                            }
                        }
                    }
                }
            }
        }
        Expr::While(cond, body) => {
            // We have to use a fixpoint iteration here to handle infinite loops that don't yield
            // We have a todolist of states to explore
            // If both the condition and body complete without yielding, we put the next state on the todo list
            // Otherwise, we yield or return the result
            let mut todo = vec![(local, global)];
            let mut visited = std::collections::HashSet::new();

            while let Some((local, global)) = todo.pop() {
                // Avoid infinite loops by tracking visited states
                if !visited.insert((local.clone(), global.clone())) {
                    continue;
                }

                // First, evaluate the condition
                for (expr_result, local1, global1) in run_expr(exprhc, cond, local, global) {
                    match expr_result {
                        ExprResult::Yielding(e) => {
                            // If condition yields, we yield the entire while expression
                            results.push((
                                ExprResult::Yielding(exprhc.while_expr(e, body.clone())),
                                local1,
                                global1,
                            ));
                        }
                        ExprResult::Returning(n) => {
                            if n != 0 {
                                // Condition is true, execute body
                                for (expr_result2, local2, global2) in
                                    run_expr(exprhc, body, local1, global1)
                                {
                                    match expr_result2 {
                                        ExprResult::Yielding(e) => {
                                            // If body yields, we yield followed by the while loop
                                            let while_expr =
                                                exprhc.while_expr(cond.clone(), body.clone());
                                            results.push((
                                                ExprResult::Yielding(
                                                    exprhc.sequence(e, while_expr),
                                                ),
                                                local2,
                                                global2,
                                            ));
                                        }
                                        ExprResult::Returning(_) => {
                                            // Body completed without yielding, continue loop
                                            todo.push((local2, global2));
                                        }
                                    }
                                }
                            } else {
                                // Condition is false, exit the loop with result 0
                                results.push((ExprResult::Returning(0), local1, global1));
                            }
                        }
                    }
                }
            }
        }
        Expr::Yield => {
            // Yield the current state
            results.push((ExprResult::Yielding(exprhc.number(0)), local, global));
        }
        Expr::Exit => {
            // Exit the whole program (kill all threads / packets)
            // Unimplemented (do we actually need this?)
            panic!("Exit not implemented");
        }
        Expr::Unknown => {
            // Returns both 0 and 1
            results.push((ExprResult::Returning(0), local.clone(), global.clone()));
            results.push((ExprResult::Returning(1), local, global));
        }
        Expr::Number(n) => {
            // Return the number directly
            results.push((ExprResult::Returning(*n), local, global));
        }
        Expr::Variable(x) => {
            // Look up the variable in local or global environment
            if is_local(x) {
                results.push((ExprResult::Returning(local.get(x)), local, global));
            } else {
                results.push((ExprResult::Returning(global.get(x)), local, global));
            }
        }
    }
    results
}

// Dummy type to represent a request for the Ser case
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum ExprRequest {
    Request,
}

impl std::fmt::Display for ExprRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Request")
    }
}

// We need a wrapper type since we can't implement Display directly for a tuple
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct LocalExpr(pub Local, pub Hc<Expr>);

impl std::fmt::Display for LocalExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

pub fn expr_to_ns(exprhc: &mut ExprHc, expr: &Hc<Expr>) -> NS<Global, LocalExpr, ExprRequest, i64> {
    // Create a new NS with an empty global environment
    let mut ns = NS::new(Global::new());

    // Track seen states to avoid duplication and infinite loops
    let mut seen_packets: HashSet<LocalExpr> = HashSet::new();
    let mut seen_globals: HashSet<Global> = HashSet::new();
    let mut todo = vec![(expr.clone(), Local::new(), Global::new())];

    // Starting state - add a request that transitions to initial state
    let initial_local = Local::new();
    let initial_expr = expr.clone();
    let initial_global = Global::new();
    let initial_local_expr = LocalExpr(initial_local.clone(), initial_expr.clone());

    // Add initial request
    ns.add_request(ExprRequest::Request, initial_local_expr.clone());
    seen_globals.insert(initial_global.clone());
    seen_packets.insert(initial_local_expr.clone());

    // Process states
    while let Some((expr, local, global)) = todo.pop() {
        let local_expr = LocalExpr(local.clone(), expr.clone());
        // Check if expr is a constant
        match expr.get() {
            Expr::Number(n) => {
                // Add a response for this local state
                ns.add_response(local_expr.clone(), *n);
            }
            _ => {
                // Get all possible results of executing this expression
                let results = run_expr(exprhc, &expr, local.clone(), global.clone());

                let mut new_globals = vec![];
                let mut new_packets = vec![];

                for (result, new_local, new_global) in results {
                    match result {
                        ExprResult::Yielding(e) => {
                            // Create a new expression to continue with
                            let new_local_expr = LocalExpr(new_local.clone(), e.clone());

                            // Add a transition from (local_expr, global) to (new_local_expr, new_global)
                            ns.add_transition(
                                local_expr.clone(),
                                global.clone(),
                                new_local_expr.clone(),
                                new_global.clone(),
                            );

                            new_globals.push(new_global.clone());
                            new_packets.push(new_local_expr.clone());
                        }
                        ExprResult::Returning(n) => {
                            // Add new global state to track if it's new
                            new_globals.push(new_global.clone());
                            let new_local_expr = LocalExpr(new_local.clone(), exprhc.number(n));
                            // Add a transition from (local_expr, global) to (new_local_expr, new_global)
                            ns.add_transition(
                                local_expr.clone(),
                                global.clone(),
                                new_local_expr.clone(),
                                new_global.clone(),
                            );
                            new_packets.push(new_local_expr.clone());
                        }
                    }
                }
                for new_global in new_globals {
                    if seen_globals.insert(new_global.clone()) {
                        // Add ALL combinations of seen packets and new global
                        for packet in seen_packets.iter() {
                            todo.push((packet.1.clone(), packet.0.clone(), new_global.clone()));
                        }
                    }
                }

                for packet in new_packets {
                    if seen_packets.insert(packet.clone()) {
                        // Add ALL combinations of seen globals and new packet
                        for global in seen_globals.iter() {
                            todo.push((packet.1.clone(), packet.0.clone(), global.clone()));
                        }
                    }
                }
            }
        }
    }

    ns
}
