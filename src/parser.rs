use hash_cons::{Hc, HcTable};
use std::fmt;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Expr {
    Assign(String, Hc<Expr>),
    Equal(Hc<Expr>, Hc<Expr>),
    Sequence(Hc<Expr>, Hc<Expr>),
    If(Hc<Expr>, Hc<Expr>, Hc<Expr>),
    While(Hc<Expr>, Hc<Expr>),
    Yield,
    Exit,
    Unknown,
    Number(i64),
    Variable(String),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Assign(var, expr) => write!(f, "{} := {}", var, expr),
            Expr::Equal(left, right) => write!(f, "{} == {}", left, right),
            Expr::Sequence(first, second) => write!(f, "{}; {}", first, second),
            Expr::If(cond, then_branch, else_branch) => {
                write!(f, "if({}){{{}}}else{{{}}}", cond, then_branch, else_branch)
            }
            Expr::While(cond, body) => write!(f, "while({}){{ {} }}", cond, body),
            Expr::Yield => write!(f, "yield"),
            Expr::Exit => write!(f, "exit"),
            Expr::Unknown => write!(f, "?"),
            Expr::Number(n) => write!(f, "{}", n),
            Expr::Variable(var) => write!(f, "{}", var),
        }
    }
}

pub struct ExprHc {
    table: HcTable<Expr>,
}

impl ExprHc {
    pub fn new() -> Self {
        Self {
            table: HcTable::new(),
        }
    }
    pub fn assign(&mut self, var: String, expr: Hc<Expr>) -> Hc<Expr> {
        self.table.hashcons(Expr::Assign(var, expr))
    }

    pub fn equal(&mut self, left: Hc<Expr>, right: Hc<Expr>) -> Hc<Expr> {
        self.table.hashcons(Expr::Equal(left, right))
    }

    pub fn sequence(&mut self, first: Hc<Expr>, second: Hc<Expr>) -> Hc<Expr> {
        self.table.hashcons(Expr::Sequence(first, second))
    }

    pub fn if_expr(
        &mut self,
        cond: Hc<Expr>,
        then_branch: Hc<Expr>,
        else_branch: Hc<Expr>,
    ) -> Hc<Expr> {
        self.table
            .hashcons(Expr::If(cond, then_branch, else_branch))
    }

    pub fn while_expr(&mut self, cond: Hc<Expr>, body: Hc<Expr>) -> Hc<Expr> {
        self.table.hashcons(Expr::While(cond, body))
    }

    pub fn yield_expr(&mut self) -> Hc<Expr> {
        self.table.hashcons(Expr::Yield)
    }

    pub fn exit(&mut self) -> Hc<Expr> {
        self.table.hashcons(Expr::Exit)
    }

    pub fn unknown(&mut self) -> Hc<Expr> {
        self.table.hashcons(Expr::Unknown)
    }

    pub fn number(&mut self, n: i64) -> Hc<Expr> {
        self.table.hashcons(Expr::Number(n))
    }

    pub fn variable(&mut self, var: String) -> Hc<Expr> {
        self.table.hashcons(Expr::Variable(var))
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(i64),
    Assign,    // :=
    Equal,     // ==
    Semicolon, // ;
    If,        // if
    Else,      // else
    While,     // while
    Yield,     // yield
    Exit,      // exit
    Question,  // ?
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    Eof,
}

/// Parse a string directly into an expression
pub fn parse(source: &str, table: &mut ExprHc) -> Result<Hc<Expr>, String> {
    let tokens = tokenize(source)?;
    let mut parser = Parser::new(tokens);
    parser.parse(table)
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self, table: &mut ExprHc) -> Result<Hc<Expr>, String> {
        let expr = self.expression(table)?;

        if self.current < self.tokens.len() && self.tokens[self.current] != Token::Eof {
            return Err(format!(
                "Unexpected token after expression: {:?}",
                self.tokens[self.current]
            ));
        }

        Ok(expr)
    }

    fn expression(&mut self, table: &mut ExprHc) -> Result<Hc<Expr>, String> {
        self.sequence(table)
    }

    fn sequence(&mut self, table: &mut ExprHc) -> Result<Hc<Expr>, String> {
        let expr = self.assignment(table)?;

        if self.match_token(&[Token::Semicolon]) {
            let right = self.expression(table)?;
            return Ok(table.sequence(expr, right));
        }

        Ok(expr)
    }

    fn assignment(&mut self, table: &mut ExprHc) -> Result<Hc<Expr>, String> {
        if let Some(Token::Identifier(name)) = self.peek() {
            let name = name.clone();
            if self.peek_next() == Some(&Token::Assign) {
                self.advance(); // consume the identifier
                self.advance(); // consume the :=
                let value = self.assignment(table)?;
                return Ok(table.assign(name, value));
            }
        }

        self.equality(table)
    }

    fn equality(&mut self, table: &mut ExprHc) -> Result<Hc<Expr>, String> {
        let mut expr = self.primary(table)?;

        if self.match_token(&[Token::Equal]) {
            let right = self.primary(table)?;
            expr = table.equal(expr, right);
        }

        Ok(expr)
    }

    fn primary(&mut self, table: &mut ExprHc) -> Result<Hc<Expr>, String> {
        let token = self.advance();

        match token {
            Some(Token::Number(n)) => Ok(table.number(*n)),
            Some(Token::Identifier(name)) => Ok(table.variable(name.clone())),
            Some(Token::Question) => Ok(table.unknown()),
            Some(Token::Yield) => Ok(table.yield_expr()),
            Some(Token::Exit) => Ok(table.exit()),
            Some(Token::If) => {
                self.consume(Token::LParen, "Expected '(' after 'if'")?;
                let condition = self.expression(table)?;
                self.consume(Token::RParen, "Expected ')' after condition")?;
                self.consume(Token::LBrace, "Expected '{' after condition")?;
                let then_branch = self.expression(table)?;
                self.consume(Token::RBrace, "Expected '}' after then branch")?;
                self.consume(Token::Else, "Expected 'else' after then branch")?;
                self.consume(Token::LBrace, "Expected '{' after 'else'")?;
                let else_branch = self.expression(table)?;
                self.consume(Token::RBrace, "Expected '}' after else branch")?;

                Ok(table.if_expr(condition, then_branch, else_branch))
            }
            Some(Token::While) => {
                self.consume(Token::LParen, "Expected '(' after 'while'")?;
                let condition = self.expression(table)?;
                self.consume(Token::RParen, "Expected ')' after condition")?;
                self.consume(Token::LBrace, "Expected '{' after condition")?;
                let body = self.expression(table)?;
                self.consume(Token::RBrace, "Expected '}' after body")?;

                Ok(table.while_expr(condition, body))
            }
            Some(Token::LParen) => {
                let expr = self.expression(table)?;
                self.consume(Token::RParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            _ => Err(format!("Unexpected token: {:?}", token)),
        }
    }

    fn match_token(&mut self, types: &[Token]) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        &self.tokens[self.current] == token_type
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.tokens[self.current] == Token::Eof
    }

    fn previous(&self) -> Option<&Token> {
        if self.current > 0 {
            Some(&self.tokens[self.current - 1])
        } else {
            None
        }
    }

    fn consume(&mut self, token_type: Token, message: &str) -> Result<&Token, String> {
        if self.check(&token_type) {
            Ok(self.advance().unwrap())
        } else {
            Err(message.to_string())
        }
    }

    fn peek(&self) -> Option<&Token> {
        if self.is_at_end() {
            None
        } else {
            Some(&self.tokens[self.current])
        }
    }

    fn peek_next(&self) -> Option<&Token> {
        if self.current + 1 >= self.tokens.len() {
            None
        } else {
            Some(&self.tokens[self.current + 1])
        }
    }
}

// Lexer implementation
pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            '0'..='9' => {
                let mut number = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() {
                        number.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(number.parse().unwrap()));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut identifier = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        identifier.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                match identifier.as_str() {
                    "if" => tokens.push(Token::If),
                    "else" => tokens.push(Token::Else),
                    "while" => tokens.push(Token::While),
                    "yield" => tokens.push(Token::Yield),
                    "exit" => tokens.push(Token::Exit),
                    _ => tokens.push(Token::Identifier(identifier)),
                }
            }
            ':' => {
                chars.next();
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::Assign);
                } else {
                    return Err("Expected '=' after ':'".to_string());
                }
            }
            '=' => {
                chars.next();
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::Equal);
                } else {
                    return Err("Expected '=' after '='".to_string());
                }
            }
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            }
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            '{' => {
                chars.next();
                tokens.push(Token::LBrace);
            }
            '}' => {
                chars.next();
                tokens.push(Token::RBrace);
            }
            '?' => {
                chars.next();
                tokens.push(Token::Question);
            }
            _ => {
                return Err(format!("Unexpected character: {}", c));
            }
        }
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tokenizer tests
    #[test]
    fn test_tokenize_assignment() {
        let tokens = tokenize("x := 42").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("x".to_string()),
                Token::Assign,
                Token::Number(42),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_tokenize_equality() {
        let tokens = tokenize("x == y").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("x".to_string()),
                Token::Equal,
                Token::Identifier("y".to_string()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_tokenize_sequence() {
        let tokens = tokenize("x := 1; y := 2").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("x".to_string()),
                Token::Assign,
                Token::Number(1),
                Token::Semicolon,
                Token::Identifier("y".to_string()),
                Token::Assign,
                Token::Number(2),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_tokenize_if_else() {
        let tokens = tokenize("if(x == 1){y := 2}else{z := 3}").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::If,
                Token::LParen,
                Token::Identifier("x".to_string()),
                Token::Equal,
                Token::Number(1),
                Token::RParen,
                Token::LBrace,
                Token::Identifier("y".to_string()),
                Token::Assign,
                Token::Number(2),
                Token::RBrace,
                Token::Else,
                Token::LBrace,
                Token::Identifier("z".to_string()),
                Token::Assign,
                Token::Number(3),
                Token::RBrace,
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_tokenize_while() {
        let tokens = tokenize("while(x == 0){x := x}").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::While,
                Token::LParen,
                Token::Identifier("x".to_string()),
                Token::Equal,
                Token::Number(0),
                Token::RParen,
                Token::LBrace,
                Token::Identifier("x".to_string()),
                Token::Assign,
                Token::Identifier("x".to_string()),
                Token::RBrace,
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_tokenize_unknown() {
        let tokens = tokenize("?").unwrap();
        assert_eq!(tokens, vec![Token::Question, Token::Eof]);
    }

    #[test]
    fn test_tokenize_number() {
        let tokens = tokenize("42").unwrap();
        assert_eq!(tokens, vec![Token::Number(42), Token::Eof]);
    }

    #[test]
    fn test_tokenize_variable() {
        let tokens = tokenize("variable").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Identifier("variable".to_string()), Token::Eof]
        );
    }

    #[test]
    fn test_tokenize_error_incomplete_assign() {
        let result = tokenize("x :");
        assert!(result.is_err());
    }

    #[test]
    fn test_tokenize_error_incomplete_equal() {
        let result = tokenize("x =");
        assert!(result.is_err());
    }

    // Parser tests
    #[test]
    fn test_parse_assignment() {
        let mut table = ExprHc::new();
        let expr = parse("x := 42", &mut table).unwrap();
        let num = table.number(42);
        let expected = table.assign("x".to_string(), num);
        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_equality() {
        let mut table = ExprHc::new();
        let expr = parse("x == y", &mut table).unwrap();
        let x_var = table.variable("x".to_string());
        let y_var = table.variable("y".to_string());
        let expected = table.equal(x_var, y_var);
        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_sequence() {
        let mut table = ExprHc::new();
        let expr = parse("x := 1; y := 2", &mut table).unwrap();
        let one = table.number(1);
        let two = table.number(2);
        let assign_x = table.assign("x".to_string(), one);
        let assign_y = table.assign("y".to_string(), two);
        let expected = table.sequence(assign_x, assign_y);
        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_if_else() {
        let mut table = ExprHc::new();
        let expr = parse("if(x == 1){y := 2}else{z := 3}", &mut table).unwrap();
        let x_var = table.variable("x".to_string());
        let one = table.number(1);
        let two = table.number(2);
        let three = table.number(3);
        let condition = table.equal(x_var, one);
        let then_branch = table.assign("y".to_string(), two);
        let else_branch = table.assign("z".to_string(), three);
        let expected = table.if_expr(condition, then_branch, else_branch);
        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_while() {
        let mut table = ExprHc::new();
        let expr = parse("while(x == 0){x := x}", &mut table).unwrap();
        let x_var = table.variable("x".to_string());
        let zero = table.number(0);
        let condition = table.equal(x_var, zero);
        let x_var2 = table.variable("x".to_string()); // Need a new instance since each is a different node
        let body = table.assign("x".to_string(), x_var2);
        let expected = table.while_expr(condition, body);
        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_unknown() {
        let mut table = ExprHc::new();
        let expr = parse("?", &mut table).unwrap();
        let expected = table.unknown();
        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_yield() {
        let mut table = ExprHc::new();
        let expr = parse("yield", &mut table).unwrap();
        let expected = table.yield_expr();
        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_exit() {
        let mut table = ExprHc::new();
        let expr = parse("exit", &mut table).unwrap();
        let expected = table.exit();
        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_number() {
        let mut table = ExprHc::new();
        let expr = parse("42", &mut table).unwrap();
        let expected = table.number(42);
        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_variable() {
        let mut table = ExprHc::new();
        let expr = parse("variable", &mut table).unwrap();
        let expected = table.variable("variable".to_string());
        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_nested_expressions() {
        let mut table = ExprHc::new();
        let expr = parse(
            "if(x == 1){if(y == 2){z := 3}else{z := 4}}else{z := 5}",
            &mut table,
        )
        .unwrap();

        let x_var = table.variable("x".to_string());
        let y_var = table.variable("y".to_string());
        let one = table.number(1);
        let two = table.number(2);
        let three = table.number(3);
        let four = table.number(4);
        let five = table.number(5);

        let inner_cond = table.equal(y_var, two);
        let inner_then = table.assign("z".to_string(), three);
        let inner_else = table.assign("z".to_string(), four);
        let inner_if = table.if_expr(inner_cond, inner_then, inner_else);

        let outer_cond = table.equal(x_var, one);
        let outer_else = table.assign("z".to_string(), five);
        let expected = table.if_expr(outer_cond, inner_if, outer_else);

        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_complex_sequence() {
        let mut table = ExprHc::new();
        let expr = parse("x := 1; y := 2; z := 3", &mut table).unwrap();

        let one = table.number(1);
        let two = table.number(2);
        let three = table.number(3);

        let assign_x = table.assign("x".to_string(), one);
        let assign_y = table.assign("y".to_string(), two);
        let assign_z = table.assign("z".to_string(), three);

        let seq_y_z = table.sequence(assign_y, assign_z);
        let expected = table.sequence(assign_x, seq_y_z);

        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_nested_while() {
        let mut table = ExprHc::new();
        let expr = parse("while(x == 0){while(y == 0){y := 1}; x := 1}", &mut table).unwrap();

        let x_var = table.variable("x".to_string());
        let y_var = table.variable("y".to_string());
        let zero = table.number(0);
        let one = table.number(1);

        let inner_cond = table.equal(y_var.clone(), zero.clone());
        let inner_body = table.assign("y".to_string(), one.clone());
        let inner_while = table.while_expr(inner_cond, inner_body);

        let assign_x = table.assign("x".to_string(), one);
        let body_seq = table.sequence(inner_while, assign_x);

        let outer_cond = table.equal(x_var.clone(), zero);
        let expected = table.while_expr(outer_cond, body_seq);

        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_complex_assignment() {
        let mut table = ExprHc::new();
        let expr = parse("x := y := 42", &mut table).unwrap();

        let num = table.number(42);
        let assign_y = table.assign("y".to_string(), num);
        let expected = table.assign("x".to_string(), assign_y);

        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_error_missing_closing_paren() {
        let mut table = ExprHc::new();
        let result = parse("if(x == 1{y := 2}else{z := 3}", &mut table);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_missing_closing_brace() {
        let mut table = ExprHc::new();
        let result = parse("if(x == 1){y := 2 else{z := 3}", &mut table);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_missing_else() {
        let mut table = ExprHc::new();
        let result = parse("if(x == 1){y := 2}{z := 3}", &mut table);
        assert!(result.is_err());
    }

    #[test]
    fn test_tokens_to_string() {
        // Test that tokens can be converted back to strings correctly via Display
        let mut table = ExprHc::new();
        let source = "if(x == 1){y := 2}else{z := 3}";
        let expr = parse(source, &mut table).unwrap();
        assert_eq!(expr.to_string(), source);
    }

    #[test]
    fn test_display_nested_expressions() {
        let mut table = ExprHc::new();
        let source = "if(x == 1){if(y == 2){z := 3}else{z := 4}}else{z := 5}";
        let expr = parse(source, &mut table).unwrap();
        assert_eq!(expr.to_string(), source);
    }

    #[test]
    fn test_roundtrip_parsing() {
        // Test that expressions can be formatted back to strings and reparsed
        let mut table = ExprHc::new();
        let original_source = "while(x == 0){y := 1; z := 2}";
        let expr = parse(original_source, &mut table).unwrap();

        let regenerated_source = expr.to_string();
        let expr2 = parse(&regenerated_source, &mut table).unwrap();

        assert_eq!(expr, expr2);
    }
}
