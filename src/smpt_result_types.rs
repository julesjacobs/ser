/// Types for SMPT results with proof/model information
/// This module defines a more structured result type to replace simple bool returns

use std::fmt;

/// A witness model showing how a property can be satisfied (when reachable)
#[derive(Debug, Clone)]
pub struct SmptModel {
    /// The raw model string from SMPT (e.g., "G___(1) RESP_bar_REQ_1(1) RESP_foo_REQ_0(1)")
    pub raw_model: String,
    /// Parsed model as place -> token count mapping
    pub place_tokens: Vec<(String, i32)>,
}

impl SmptModel {
    /// Parse a raw SMPT model string into structured data
    pub fn parse(raw_model: String) -> Self {
        let mut place_tokens = Vec::new();
        
        // Parse the model format: "place_name(token_count) place_name2(token_count2) ..."
        for part in raw_model.split_whitespace() {
            if let Some(open_paren) = part.find('(') {
                if let Some(close_paren) = part.find(')') {
                    let place_name = part[..open_paren].to_string();
                    if let Ok(token_count) = part[open_paren + 1..close_paren].parse::<i32>() {
                        place_tokens.push((place_name, token_count));
                    }
                }
            }
        }
        
        SmptModel {
            raw_model,
            place_tokens,
        }
    }
}

impl fmt::Display for SmptModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Model: {}", self.raw_model)
    }
}

/// A proof certificate showing why a property cannot be satisfied (when unreachable)
#[derive(Debug, Clone)]
pub struct SmptProof {
    /// The raw proof string from SMPT
    pub raw_proof: String,
    /// The method that generated this proof (BMC, PDR, etc.)
    pub method: Option<String>,
    /// Whether the proof was verified
    pub verified: bool,
}

impl fmt::Display for SmptProof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Proof ({}): {}", 
               self.method.as_deref().unwrap_or("unknown"), 
               self.raw_proof)
    }
}

/// Result of SMPT verification with optional proof/model information
#[derive(Debug, Clone)]
pub enum SmptVerificationResult {
    /// Property is reachable/satisfiable (with optional witness model)
    Reachable { 
        model: Option<SmptModel>,
        execution_time: Option<u64>,
        method_used: Option<String>,
        raw_stdout: String,
        raw_stderr: String,
    },
    /// Property is unreachable/unsatisfiable (with optional proof certificate)
    Unreachable { 
        proof: Option<SmptProof>,
        execution_time: Option<u64>,
        method_used: Option<String>,
        raw_stdout: String,
        raw_stderr: String,
    },
}

impl SmptVerificationResult {
    /// Returns true if the property is reachable
    pub fn is_reachable(&self) -> bool {
        matches!(self, SmptVerificationResult::Reachable { .. })
    }
    
    /// Returns true if the property is unreachable  
    pub fn is_unreachable(&self) -> bool {
        matches!(self, SmptVerificationResult::Unreachable { .. })
    }
    
    /// Get the execution time if available
    pub fn execution_time(&self) -> Option<u64> {
        match self {
            SmptVerificationResult::Reachable { execution_time, .. } => *execution_time,
            SmptVerificationResult::Unreachable { execution_time, .. } => *execution_time,
        }
    }
    
    /// Get the method used if available
    pub fn method_used(&self) -> Option<&str> {
        match self {
            SmptVerificationResult::Reachable { method_used, .. } => method_used.as_deref(),
            SmptVerificationResult::Unreachable { method_used, .. } => method_used.as_deref(),
        }
    }
    
    /// Get the raw stdout
    pub fn raw_stdout(&self) -> &str {
        match self {
            SmptVerificationResult::Reachable { raw_stdout, .. } => raw_stdout,
            SmptVerificationResult::Unreachable { raw_stdout, .. } => raw_stdout,
        }
    }
    
    /// Get the raw stderr
    pub fn raw_stderr(&self) -> &str {
        match self {
            SmptVerificationResult::Reachable { raw_stderr, .. } => raw_stderr,
            SmptVerificationResult::Unreachable { raw_stderr, .. } => raw_stderr,
        }
    }
    
    /// Get the model if this is a reachable result
    pub fn model(&self) -> Option<&SmptModel> {
        match self {
            SmptVerificationResult::Reachable { model, .. } => model.as_ref(),
            _ => None,
        }
    }
    
    /// Get the proof if this is an unreachable result
    pub fn proof(&self) -> Option<&SmptProof> {
        match self {
            SmptVerificationResult::Unreachable { proof, .. } => proof.as_ref(),
            _ => None,
        }
    }
}

impl fmt::Display for SmptVerificationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SmptVerificationResult::Reachable { model, method_used, .. } => {
                write!(f, "REACHABLE")?;
                if let Some(method) = method_used {
                    write!(f, " ({})", method)?;
                }
                if let Some(m) = model {
                    write!(f, " - {}", m)?;
                }
                Ok(())
            }
            SmptVerificationResult::Unreachable { proof, method_used, .. } => {
                write!(f, "UNREACHABLE")?;
                if let Some(method) = method_used {
                    write!(f, " ({})", method)?;
                }
                if let Some(p) = proof {
                    write!(f, " - {}", p)?;
                }
                Ok(())
            }
        }
    }
}

/// Error types for SMPT verification
#[derive(Debug, Clone)]
pub enum SmptError {
    /// SMPT tool execution failed
    ExecutionError(String),
    /// SMPT output could not be parsed
    ParseError(String),
    /// SMPT tool is not available/installed
    ToolNotAvailable(String),
    /// Timeout during verification
    Timeout(u64), // timeout in milliseconds
}

impl fmt::Display for SmptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SmptError::ExecutionError(msg) => write!(f, "SMPT execution error: {}", msg),
            SmptError::ParseError(msg) => write!(f, "SMPT output parse error: {}", msg),
            SmptError::ToolNotAvailable(msg) => write!(f, "SMPT tool not available: {}", msg),
            SmptError::Timeout(ms) => write!(f, "SMPT timeout after {}ms", ms),
        }
    }
}

impl std::error::Error for SmptError {}

/// The main result type for SMPT operations
pub type SmptResult = Result<SmptVerificationResult, SmptError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_parsing() {
        let model = SmptModel::parse("G___(1) RESP_bar_REQ_1(1) RESP_foo_REQ_0(1)".to_string());
        assert_eq!(model.place_tokens.len(), 3);
        assert_eq!(model.place_tokens[0], ("G___".to_string(), 1));
        assert_eq!(model.place_tokens[1], ("RESP_bar_REQ_1".to_string(), 1));
        assert_eq!(model.place_tokens[2], ("RESP_foo_REQ_0".to_string(), 1));
    }
    
    #[test]
    fn test_result_accessors() {
        let model = SmptModel::parse("G___(1)".to_string());
        let result = SmptVerificationResult::Reachable {
            model: Some(model),
            execution_time: Some(100),
            method_used: Some("BMC".to_string()),
            raw_stdout: "test".to_string(),
            raw_stderr: "".to_string(),
        };
        
        assert!(result.is_reachable());
        assert!(!result.is_unreachable());
        assert_eq!(result.execution_time(), Some(100));
        assert_eq!(result.method_used(), Some("BMC"));
        assert!(result.model().is_some());
        assert!(result.proof().is_none());
    }
}