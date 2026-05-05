use std::{fmt::Display, slice::Iter};

use crate::asm::{self};
use crate::lex::{Keyword, Token};

#[derive(Debug, PartialEq)]
pub enum Exp {
    Constant(i32),
    Unary(UnaryOperator, Box<Exp>),
}

#[derive(Debug, PartialEq)]
pub enum UnaryOperator {
    Complement,
    Negate,
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Return(Exp),
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Return(exp) => write!(f, "Return(\n          {:?}\n        )", exp),
        }
    }
}

impl Statement {
    pub fn to_asm(&self, instructions: &mut Vec<asm::Instruction>) {
        match self {
            Self::Return(exp) => {
                let left_operand = match exp {
                    Exp::Constant(val) => asm::Operand::Imm(*val),
                    Exp::Unary(_, _) => asm::Operand::Imm(1),
                };
                instructions.push(asm::Instruction::Mov {
                    source: left_operand,
                    dest: asm::Operand::Register,
                });
                instructions.push(asm::Instruction::Ret);
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FunctionDefinition {
    Function(String, Statement),
}

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function(name, statement) => {
                write!(f, "name=\"{name}\",\n        body={statement}")
            }
        }
    }
}

impl FunctionDefinition {
    pub fn to_asm(&self) -> asm::FunctionDefinition {
        match self {
            Self::Function(name, statement) => {
                let mut instructions = vec![];
                statement.to_asm(&mut instructions);
                asm::FunctionDefinition::Function(name.clone(), instructions)
            }
        }
    }
}

#[derive(Debug)]
pub struct Program {
    function: FunctionDefinition,
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\
Program(
  Function(
    {}
  )
)\
",
            self.function
        )
    }
}

impl Program {
    pub fn to_asm(&self) -> asm::Program {
        let function = self.function.to_asm();
        asm::Program { function }
    }
}

pub fn parse_program(tokens: &mut Iter<Token>) -> Result<Program, String> {
    let function = parse_function(tokens)?;
    if let Some(_) = tokens.next() {
        Err("extra tokens at the end of the file".to_string())
    } else {
        Ok(Program { function })
    }
}

pub fn parse_function(tokens: &mut Iter<Token>) -> Result<FunctionDefinition, String> {
    expect(&Token::Keyword(Keyword::Int), tokens)?;
    let ident = parse_ident(tokens)?;
    expect(&Token::OpenParenthesis, tokens)?;
    expect_keyword(&Keyword::Void, tokens)?;
    expect(&Token::CloseParenthesis, tokens)?;
    expect(&Token::OpenBrace, tokens)?;
    let statement = parse_statement(tokens)?;
    expect(&Token::CloseBrace, tokens)?;
    Ok(FunctionDefinition::Function(ident, statement))
}

fn parse_unop(token: &Token) -> Result<UnaryOperator, String> {
    match token {
        Token::Hyphen => Ok(UnaryOperator::Negate),
        Token::Tilde => Ok(UnaryOperator::Complement),
        unexpected => Err(format!("Expected '<unary>' but found '{}'", unexpected)),
    }
}

pub fn parse_exp(tokens: &mut Iter<Token>) -> Result<Exp, String> {
    match tokens.next() {
        Some(Token::Constant(val)) => Ok(Exp::Constant(*val)),
        Some(token) if token == &Token::Hyphen || token == &Token::Tilde => {
            let unary_op = parse_unop(token)?;
            let inner_exp = parse_exp(tokens)?;
            Ok(Exp::Unary(unary_op, Box::new(inner_exp)))
        }
        Some(Token::OpenParenthesis) => {
            let exp = parse_exp(tokens)?;
            expect(&Token::CloseParenthesis, tokens)?;
            Ok(exp)
        }
        None => Err("Expected '<exp>' but reached the end".to_string()),
        Some(unexpected) => Err(format!("Expected '<exp>' but found '{}'", unexpected)),
    }
}

pub fn parse_statement(tokens: &mut Iter<Token>) -> Result<Statement, String> {
    expect(&Token::Keyword(Keyword::Return), tokens)?;
    let exp = parse_exp(tokens)?;
    expect(&Token::SemiColon, tokens)?;

    Ok(Statement::Return(exp))
}

fn parse_ident(tokens: &mut Iter<Token>) -> Result<String, String> {
    if let Some(actual) = tokens.next() {
        if let Token::Identifier(ident) = actual {
            Ok(ident.clone())
        } else {
            Err(format!("Expected <identifier> but found '{}'", actual))
        }
    } else {
        Err(format!("Expected <identifier> but reached the end"))
    }
}

fn expect_keyword(expected_keyword: &Keyword, tokens: &mut Iter<Token>) -> Result<(), String> {
    if let Some(actual) = tokens.next() {
        if let Token::Keyword(keyword) = actual {
            if keyword != expected_keyword {
                Err(format!(
                    "Expected '{expected_keyword}' but found '{actual}'",
                ))
            } else {
                Ok(())
            }
        } else {
            Err(format!(
                "Expected '{expected_keyword}' but found '{actual}'",
            ))
        }
    } else {
        Err(format!("Expected '{expected_keyword}' but reached the end"))
    }
}

fn expect(expected_token: &Token, tokens: &mut Iter<Token>) -> Result<(), String> {
    if let Some(actual) = tokens.next() {
        if actual != expected_token {
            Err(format!(
                "Expected '{}' but found '{actual}'",
                expected_token
            ))
        } else {
            Ok(())
        }
    } else {
        Err(format!("Expected '{}' but reached the end", expected_token))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::{
            Exp, FunctionDefinition, Statement, UnaryOperator, parse_exp, parse_function,
            parse_statement,
        },
        lex::{
            Keyword::{Int, Return, Void},
            Token::{
                CloseBrace, CloseParenthesis, Constant, Hyphen, Identifier, Keyword, OpenBrace,
                OpenParenthesis, SemiColon, Tilde, TwoHyphens,
            },
        },
    };

    #[test]
    fn valid_statement() {
        let tokens = vec![Keyword(Return), Constant(2), SemiColon];
        let statement = parse_statement(&mut tokens.iter()).unwrap();
        assert_eq!(statement, Statement::Return(crate::ast::Exp::Constant(2)));
    }
    #[test]
    fn invalid_statement() {
        let tokens = vec![Keyword(Return), Constant(2)];

        if let Err(msg) = parse_statement(&mut tokens.iter()) {
            assert_eq!(msg, "Expected ';' but reached the end");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn invalid_expression() {
        let tokens = vec![Keyword(Return)];
        if let Err(msg) = parse_exp(&mut tokens.iter()) {
            assert_eq!(msg, "Expected '<exp>' but found 'Keyword(return)'");
        } else {
            panic!("expected an error here");
        }
    }
    #[test]
    fn missing_expression() {
        let tokens = vec![];
        if let Err(msg) = parse_exp(&mut tokens.iter()) {
            assert_eq!(msg, "Expected '<exp>' but reached the end");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn unary_hyphen_missing_expression() {
        let tokens = vec![Hyphen];
        if let Err(msg) = parse_exp(&mut tokens.iter()) {
            assert_eq!(msg, "Expected '<exp>' but reached the end");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn unary_tilde_missing_expression() {
        let tokens = vec![Tilde];
        if let Err(msg) = parse_exp(&mut tokens.iter()) {
            assert_eq!(msg, "Expected '<exp>' but reached the end");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn unary_hyphen_valid_expression() {
        let tokens = vec![Hyphen, Constant(4)];
        let res = parse_exp(&mut tokens.iter()).unwrap();
        assert_eq!(
            res,
            Exp::Unary(
                crate::ast::UnaryOperator::Negate,
                Box::new(Exp::Constant(4))
            )
        );
    }

    #[test]
    fn expression_parenthesis_missing_expression() {
        let tokens = vec![OpenParenthesis];
        if let Err(msg) = parse_exp(&mut tokens.iter()) {
            assert_eq!(msg, "Expected '<exp>' but reached the end");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn expression_parenthesis_missing_close() {
        let tokens = vec![OpenParenthesis, Constant(5)];
        if let Err(msg) = parse_exp(&mut tokens.iter()) {
            assert_eq!(msg, "Expected ')' but reached the end");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn expression_wrapped_valid() {
        let tokens = vec![
            Hyphen,
            OpenParenthesis,
            Hyphen,
            Constant(6),
            CloseParenthesis,
        ];
        let res = parse_exp(&mut tokens.iter()).unwrap();
        assert_eq!(
            res,
            Exp::Unary(
                UnaryOperator::Negate,
                Box::new(Exp::Unary(
                    UnaryOperator::Negate,
                    Box::new(Exp::Constant(6))
                ))
            )
        );
    }

    #[test]
    fn function_missing_int_keyword() {
        let tokens = vec![Identifier("main".to_string())];
        if let Err(msg) = parse_function(&mut tokens.iter()) {
            assert_eq!(msg, "Expected 'Keyword(int)' but found 'Identifier(main)'");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn function_missing_identifier() {
        let tokens = vec![Keyword(Int), Keyword(Int)];

        if let Err(msg) = parse_function(&mut tokens.iter()) {
            assert_eq!(msg, "Expected <identifier> but found 'Keyword(int)'");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn function_missing_open_parenthesis() {
        let tokens = vec![Keyword(Int), Identifier("thing".to_string()), Keyword(Int)];

        if let Err(msg) = parse_function(&mut tokens.iter()) {
            assert_eq!(msg, "Expected '(' but found 'Keyword(int)'");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn function_missing_void() {
        let tokens = vec![
            Keyword(Int),
            Identifier("thing".to_string()),
            OpenParenthesis,
            OpenParenthesis,
        ];

        if let Err(msg) = parse_function(&mut tokens.iter()) {
            assert_eq!(msg, "Expected 'void' but found '('");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn function_missing_close_parenthesis() {
        let tokens = vec![
            Keyword(Int),
            Identifier("thing".to_string()),
            OpenParenthesis,
            Keyword(Void),
            OpenParenthesis,
        ];

        if let Err(msg) = parse_function(&mut tokens.iter()) {
            assert_eq!(msg, "Expected ')' but found '('");
        } else {
            panic!("expected an error here");
        }
    }
    #[test]
    fn function_missing_open_brace() {
        let tokens = vec![
            Keyword(Int),
            Identifier("thing".to_string()),
            OpenParenthesis,
            Keyword(Void),
            CloseParenthesis,
            CloseParenthesis,
        ];

        if let Err(msg) = parse_function(&mut tokens.iter()) {
            assert_eq!(msg, "Expected '{' but found ')'");
        } else {
            panic!("expected an error here");
        }
    }
    #[test]
    fn function_missing_close_brace() {
        let tokens = vec![
            Keyword(Int),
            Identifier("thing".to_string()),
            OpenParenthesis,
            Keyword(Void),
            CloseParenthesis,
            OpenBrace,
            Keyword(Return),
            Constant(2),
            SemiColon,
            OpenBrace,
        ];

        if let Err(msg) = parse_function(&mut tokens.iter()) {
            assert_eq!(msg, "Expected '}' but found '{'");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn function_missing_statement() {
        let tokens = vec![
            Keyword(Int),
            Identifier("thing".to_string()),
            OpenParenthesis,
            Keyword(Void),
            CloseParenthesis,
            OpenBrace,
            OpenBrace,
        ];

        if let Err(msg) = parse_function(&mut tokens.iter()) {
            assert_eq!(msg, "Expected 'Keyword(return)' but found '{'");
        } else {
            panic!("expected an error here");
        }
    }
    #[test]
    fn valid_function() {
        let tokens = vec![
            Keyword(Int),
            Identifier("thing".to_string()),
            OpenParenthesis,
            Keyword(Void),
            CloseParenthesis,
            OpenBrace,
            Keyword(Return),
            Constant(4),
            SemiColon,
            CloseBrace,
        ];

        let function_definition = parse_function(&mut tokens.iter()).unwrap();
        assert_eq!(
            function_definition,
            FunctionDefinition::Function(
                "thing".to_string(),
                Statement::Return(crate::ast::Exp::Constant(4))
            )
        );
    }
}
