#![allow(dead_code)]

mod ns;
mod parser;
mod kleene;
mod semilinear;

use parser::{Expr, ExprHc, parse};
use ns::NS;

fn main() {
    let test_cases = vec![
        "x := 42",
        "x == y",
        "x := 1; y := 2",
        "if(x == 1){y := 2}else{z := 3}",
        "while(x == 0){x := x; y := 2}",
        "yield",
        "exit",
        "?",
        "42",
        "variable",
    ];

    let mut table = ExprHc::new();

    for source in test_cases {
        println!("Source: {}", source);
        match parse(source, &mut table) {
            Ok(expr) => println!("AST: {:?}\nPrinted: {}\n", expr, expr),
            Err(e) => println!("Parse error: {}\n", e),
        }
    }

    // Demonstrate hash consing
    println!("\nDemonstrating hash consing:");
    let mut table = ExprHc::new();
    let expr1 = parse("x := 42", &mut table).unwrap();
    let expr2 = parse("x := 42", &mut table).unwrap();

    println!("expr1 == expr2: {}", expr1 == expr2);

    let complex = parse("if(x == 1){y := 42}else{z := 42}", &mut table).unwrap();

    // Find the 42 constants
    let number1 = match complex.as_ref() {
        Expr::If(_, then_branch, _) => match then_branch.as_ref() {
            Expr::Assign(_, num) => num,
            _ => panic!("Expected Assign in then branch"),
        },
        _ => panic!("Expected If expression"),
    };

    let number2 = match complex.as_ref() {
        Expr::If(_, _, else_branch) => match else_branch.as_ref() {
            Expr::Assign(_, num) => num,
            _ => panic!("Expected Assign in else branch"),
        },
        _ => panic!("Expected If expression"),
    };

    println!(
        "Same numbers in different branches are the same object: {}",
        std::ptr::eq(number1, number2)
    );

    // Demonstrate Network System with GraphViz visualization
    println!("\n--- Network System GraphViz Demo ---");

    // Create a simple network system for a login flow
    let mut ns = NS::<String, String, String, String>::new();

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

    // Generate and print GraphViz DOT representations
    let dot = ns.to_graphviz();

    println!("Full Network System Visualization (DOT format):");
    println!("{}", dot);

    // Save DOT files, generate visualizations, and automatically open them
    println!("\nSaving and opening visualizations...");
    match ns.save_graphviz("login_flow", true) {
        Ok(files) => {
            println!("Successfully generated the following files:");
            for file in files {
                println!("- {}", file);
            }
            println!("\nOpened PNG files in your default image viewer.");
        },
        Err(e) => {
            println!("Failed to save visualizations: {}", e);
            println!("\nYou can still visualize the DOT format manually:");
            println!("1. Save the DOT content above to a file (e.g., network.dot)");
            println!("2. Use Graphviz: 'dot -Tpng network.dot -o network.png'");
            println!("3. Or use an online GraphViz viewer like https://dreampuf.github.io/GraphvizOnline/");
        }
    }
}
