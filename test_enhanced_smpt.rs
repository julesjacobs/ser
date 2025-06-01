use std::env;

mod smpt;
mod smpt_result_types;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <net_file> <xml_file>", args[0]);
        std::process::exit(1);
    }
    
    let net_file = &args[1];
    let xml_file = &args[2];
    
    println!("Testing enhanced SMPT with:");
    println!("  Net file: {}", net_file);
    println!("  XML file: {}", xml_file);
    println!();
    
    match smpt::run_smpt_enhanced(net_file, xml_file, Some(30), None) {
        Ok(result) => {
            println!("✅ SMPT Success!");
            println!("Result: {}", result);
            println!();
            
            match result {
                smpt_result_types::SmptVerificationResult::Reachable { model, .. } => {
                    if let Some(m) = model {
                        println!("📊 Model Details:");
                        for (place, tokens) in &m.place_tokens {
                            println!("  {}: {} tokens", place, tokens);
                        }
                    }
                }
                smpt_result_types::SmptVerificationResult::Unreachable { proof, .. } => {
                    if let Some(p) = proof {
                        println!("🔒 Proof Details:");
                        println!("  Method: {:?}", p.method);
                        println!("  Verified: {}", p.verified);
                        println!("  Proof: {}", p.raw_proof);
                    }
                }
            }
            
            println!();
            println!("📋 Raw Output:");
            println!("stdout: {}", result.raw_stdout());
            println!("stderr: {}", result.raw_stderr());
        }
        Err(e) => {
            println!("❌ SMPT Error: {}", e);
        }
    }
}