use std::{fmt::Display, slice::Iter};

use crate::error::DccError;
use crate::lex::{Keyword, Token};

#[derive(Debug, PartialEq)]
pub enum Exp {
    Constant(i32),
    Unary(UnaryOperator, Box<Exp>),
    Binary(BinaryOperator, Box<Exp>, Box<Exp>),
}

#[derive(Debug, PartialEq)]
pub enum UnaryOperator {
    Complement,
    Negate,
}

#[derive(Debug, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
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

#[derive(Debug)]
pub struct Program {
    pub function: FunctionDefinition,
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

pub fn parse_program(tokens: &mut Iter<Token>) -> Result<Program, DccError> {
    let function = parse_function(tokens)?;
    if let Some(_) = tokens.next() {
        Err(DccError::ExtraTokens)
    } else {
        Ok(Program { function })
    }
}

pub fn parse_function(tokens: &mut Iter<Token>) -> Result<FunctionDefinition, DccError> {
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

fn parse_unop(token: &Token) -> Result<UnaryOperator, DccError> {
    match token {
        Token::Hyphen => Ok(UnaryOperator::Negate),
        Token::Tilde => Ok(UnaryOperator::Complement),
        unexpected => Err(DccError::ExpectedToken {
            actual: unexpected.clone(),
            expected: "<unary>".into(),
        }),
    }
}

pub fn parse_exp(tokens: &mut Iter<Token>) -> Result<Exp, DccError> {
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
        None => Err(DccError::ExpectedMoreTokens {
            expected: "<exp>".into(),
        }),
        Some(unexpected) => Err(DccError::ExpectedToken {
            actual: unexpected.clone(),
            expected: "<exp>".into(),
        }),
    }
}

pub fn parse_statement(tokens: &mut Iter<Token>) -> Result<Statement, DccError> {
    expect(&Token::Keyword(Keyword::Return), tokens)?;
    let exp = parse_exp(tokens)?;
    expect(&Token::SemiColon, tokens)?;

    Ok(Statement::Return(exp))
}

fn parse_ident(tokens: &mut Iter<Token>) -> Result<String, DccError> {
    if let Some(actual) = tokens.next() {
        if let Token::Identifier(ident) = actual {
            Ok(ident.clone())
        } else {
            Err(DccError::ExpectedToken {
                actual: actual.clone(),
                expected: "<identifier>".into(),
            })
        }
    } else {
        Err(DccError::ExpectedMoreTokens {
            expected: "<identifier>".into(),
        })
    }
}

fn expect_keyword(expected_keyword: &Keyword, tokens: &mut Iter<Token>) -> Result<(), DccError> {
    if let Some(actual) = tokens.next() {
        if let Token::Keyword(keyword) = actual {
            if keyword != expected_keyword {
                Err(DccError::ExpectedKeyword {
                    actual: actual.clone(),
                    expected: expected_keyword.clone(),
                })
            } else {
                Ok(())
            }
        } else {
            Err(DccError::ExpectedKeyword {
                actual: actual.clone(),
                expected: expected_keyword.clone(),
            })
        }
    } else {
        Err(DccError::ExpectedMoreKeywords {
            expected: expected_keyword.clone(),
        })
    }
}

fn expect(expected_token: &Token, tokens: &mut Iter<Token>) -> Result<(), DccError> {
    if let Some(actual) = tokens.next() {
        if actual != expected_token {
            Err(DccError::ExpectedToken {
                actual: actual.clone(),
                expected: expected_token.to_string(),
            })
        } else {
            Ok(())
        }
    } else {
        Err(DccError::ExpectedMoreTokens {
            expected: expected_token.to_string(),
        })
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
                OpenParenthesis, SemiColon, Tilde,
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

        if let Err(err) = parse_statement(&mut tokens.iter()) {
            assert_eq!(err.to_string(), "Expected ';' but reached the end");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn invalid_expression() {
        let tokens = vec![Keyword(Return)];
        if let Err(err) = parse_exp(&mut tokens.iter()) {
            assert_eq!(
                err.to_string(),
                "Expected '<exp>' but found 'Keyword(return)'"
            );
        } else {
            panic!("expected an error here");
        }
    }
    #[test]
    fn missing_expression() {
        let tokens = vec![];
        if let Err(err) = parse_exp(&mut tokens.iter()) {
            assert_eq!(err.to_string(), "Expected '<exp>' but reached the end");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn unary_hyphen_missing_expression() {
        let tokens = vec![Hyphen];
        if let Err(err) = parse_exp(&mut tokens.iter()) {
            assert_eq!(err.to_string(), "Expected '<exp>' but reached the end");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn unary_tilde_missing_expression() {
        let tokens = vec![Tilde];
        if let Err(err) = parse_exp(&mut tokens.iter()) {
            assert_eq!(err.to_string(), "Expected '<exp>' but reached the end");
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
        if let Err(err) = parse_exp(&mut tokens.iter()) {
            assert_eq!(err.to_string(), "Expected '<exp>' but reached the end");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn expression_parenthesis_missing_close() {
        let tokens = vec![OpenParenthesis, Constant(5)];
        if let Err(err) = parse_exp(&mut tokens.iter()) {
            assert_eq!(err.to_string(), "Expected ')' but reached the end");
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
        if let Err(err) = parse_function(&mut tokens.iter()) {
            assert_eq!(
                err.to_string(),
                "Expected 'Keyword(int)' but found 'Identifier(main)'"
            );
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn function_missing_identifier() {
        let tokens = vec![Keyword(Int), Keyword(Int)];

        if let Err(err) = parse_function(&mut tokens.iter()) {
            assert_eq!(
                err.to_string(),
                "Expected '<identifier>' but found 'Keyword(int)'"
            );
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn function_missing_open_parenthesis() {
        let tokens = vec![Keyword(Int), Identifier("thing".to_string()), Keyword(Int)];

        if let Err(err) = parse_function(&mut tokens.iter()) {
            assert_eq!(err.to_string(), "Expected '(' but found 'Keyword(int)'");
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

        if let Err(err) = parse_function(&mut tokens.iter()) {
            assert_eq!(err.to_string(), "Expected 'void' but found '('");
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

        if let Err(err) = parse_function(&mut tokens.iter()) {
            assert_eq!(err.to_string(), "Expected ')' but found '('");
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

        if let Err(err) = parse_function(&mut tokens.iter()) {
            assert_eq!(err.to_string(), "Expected '{' but found ')'");
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

        if let Err(err) = parse_function(&mut tokens.iter()) {
            assert_eq!(err.to_string(), "Expected '}' but found '{'");
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

        if let Err(err) = parse_function(&mut tokens.iter()) {
            assert_eq!(err.to_string(), "Expected 'Keyword(return)' but found '{'");
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
