use std::iter::Peekable;
use std::{fmt::Display, slice::Iter};

use crate::error::DccError;
use crate::lex::{Keyword, Token};

#[derive(Debug, PartialEq)]
pub enum Exp {
    Constant(i32),
    Binary(BinaryOperator, Box<Exp>, Box<Exp>),
    Unary(UnaryOperator, Box<Exp>),
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

pub fn parse_program(tokens: &mut Peekable<Iter<Token>>) -> Result<Program, DccError> {
    let function = parse_function(tokens)?;
    if let Some(_) = tokens.next() {
        Err(DccError::ExtraTokens)
    } else {
        Ok(Program { function })
    }
}

pub fn parse_function(tokens: &mut Peekable<Iter<Token>>) -> Result<FunctionDefinition, DccError> {
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

fn parse_binop(tokens: &mut Peekable<Iter<Token>>) -> Result<BinaryOperator, DccError> {
    match tokens.next() {
        Some(Token::Plus) => Ok(BinaryOperator::Add),
        Some(Token::Hyphen) => Ok(BinaryOperator::Subtract),
        Some(Token::Asterisk) => Ok(BinaryOperator::Multiply),
        Some(Token::ForwardSlash) => Ok(BinaryOperator::Divide),
        Some(Token::Percent) => Ok(BinaryOperator::Remainder),
        Some(other) => Err(DccError::ExpectedToken {
            actual: other.clone(),
            expected: "<op>".into(),
        }),
        None => Err(DccError::ExpectedMoreTokens {
            expected: "<op>".into(),
        }),
    }
}

pub fn parse_factor(tokens: &mut Peekable<Iter<Token>>) -> Result<Exp, DccError> {
    match tokens.next() {
        Some(Token::Constant(val)) => Ok(Exp::Constant(*val)),
        Some(token) if token == &Token::Hyphen || token == &Token::Tilde => {
            let unary_op = parse_unop(token)?;
            let inner_exp = parse_factor(tokens)?;
            Ok(Exp::Unary(unary_op, Box::new(inner_exp)))
        }
        Some(Token::OpenParenthesis) => {
            let exp = parse_exp(tokens, 0)?;
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

fn get_operator_precedence(token: &Token) -> i32 {
    match token {
        Token::Plus => 45,
        Token::Hyphen => 45,
        Token::Asterisk => 50,
        Token::ForwardSlash => 50,
        Token::Percent => 50,
        _ => -1,
    }
}

pub fn parse_exp(tokens: &mut Peekable<Iter<Token>>, min_prec: i32) -> Result<Exp, DccError> {
    let mut left = parse_factor(tokens)?;
    while match tokens.peek() {
        Some(token) if get_operator_precedence(*token) >= min_prec => true,
        _ => false,
    } {
        let token = *tokens.peek().unwrap();
        let operator = parse_binop(tokens)?;

        let right = parse_exp(tokens, get_operator_precedence(token) + 1)?;
        left = Exp::Binary(operator, Box::new(left), Box::new(right));
    }
    Ok(left)
}

pub fn parse_statement(tokens: &mut Peekable<Iter<Token>>) -> Result<Statement, DccError> {
    expect(&Token::Keyword(Keyword::Return), tokens)?;
    let exp = parse_exp(tokens, 0)?;
    expect(&Token::SemiColon, tokens)?;

    Ok(Statement::Return(exp))
}

fn parse_ident(tokens: &mut Peekable<Iter<Token>>) -> Result<String, DccError> {
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

fn expect_keyword(
    expected_keyword: &Keyword,
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<(), DccError> {
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

fn expect(expected_token: &Token, tokens: &mut Peekable<Iter<Token>>) -> Result<(), DccError> {
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
            BinaryOperator, Exp, FunctionDefinition, Statement, UnaryOperator, parse_exp,
            parse_factor, parse_function, parse_statement,
        },
        lex::{
            Keyword::{Int, Return, Void},
            Token::{
                self, CloseBrace, CloseParenthesis, Constant, Hyphen, Identifier, Keyword,
                OpenBrace, OpenParenthesis, Plus, SemiColon, Tilde,
            },
        },
    };

    #[test]
    fn valid_statement() {
        let tokens = vec![Keyword(Return), Constant(2), SemiColon];
        let statement = parse_statement(&mut tokens.iter().peekable()).unwrap();
        assert_eq!(statement, Statement::Return(Exp::Constant(2)));
    }

    #[test]
    fn invalid_statement() {
        let tokens = vec![Keyword(Return), Constant(2)];

        if let Err(err) = parse_statement(&mut tokens.iter().peekable()) {
            assert_eq!(err.to_string(), "Expected ';' but reached the end");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn invalid_expression() {
        let tokens = vec![Keyword(Return)];
        if let Err(err) = parse_exp(&mut tokens.iter().peekable(), 0) {
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
        if let Err(err) = parse_exp(&mut tokens.iter().peekable(), 0) {
            assert_eq!(err.to_string(), "Expected '<exp>' but reached the end");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn unary_hyphen_missing_expression() {
        let tokens = vec![Hyphen];
        if let Err(err) = parse_factor(&mut tokens.iter().peekable()) {
            assert_eq!(err.to_string(), "Expected '<exp>' but reached the end");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn unary_with_binop() {
        let tokens = vec![
            Tilde,
            OpenParenthesis,
            Constant(1),
            Plus,
            Constant(2),
            CloseParenthesis,
        ];
        let res = parse_exp(&mut tokens.iter().peekable(), 0).unwrap();
        assert_eq!(
            res,
            Exp::Unary(
                UnaryOperator::Complement,
                Box::new(Exp::Binary(
                    BinaryOperator::Add,
                    Box::new(Exp::Constant(1)),
                    Box::new(Exp::Constant(2))
                ))
            )
        );
    }

    #[test]
    fn unary_tilde_missing_expression() {
        let tokens = vec![Tilde];
        if let Err(err) = parse_factor(&mut tokens.iter().peekable()) {
            assert_eq!(err.to_string(), "Expected '<exp>' but reached the end");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn constant_valid_factor() {
        let tokens = vec![Constant(5)];
        let res = parse_exp(&mut tokens.iter().peekable(), 0).unwrap();
        assert_eq!(res, Exp::Constant(5));
    }

    #[test]
    fn unary_hyphen_valid_expression() {
        let tokens = vec![Hyphen, Constant(4)];
        let res = parse_factor(&mut tokens.iter().peekable()).unwrap();
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
        if let Err(err) = parse_factor(&mut tokens.iter().peekable()) {
            assert_eq!(err.to_string(), "Expected '<exp>' but reached the end");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn factor_missing_tokens() {
        let tokens = vec![];
        if let Err(err) = parse_factor(&mut tokens.iter().peekable()) {
            assert_eq!(err.to_string(), "Expected '<exp>' but reached the end");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn expression_parenthesis_missing_close() {
        let tokens = vec![OpenParenthesis, Constant(5)];
        if let Err(err) = parse_factor(&mut tokens.iter().peekable()) {
            assert_eq!(err.to_string(), "Expected ')' but reached the end");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn binary_expression_missing_right() {
        let tokens = vec![Constant(1), Token::Plus];
        if let Err(err) = parse_exp(&mut tokens.iter().peekable(), 0) {
            assert_eq!(err.to_string(), "Expected '<exp>' but reached the end");
        } else {
            panic!("expected an error here");
        }
    }

    #[test]
    fn binary_precedence() {
        // (1 + (2 / 3)) - (4 * 5)
        let tokens = vec![
            Constant(1),
            Token::Plus,
            Constant(2),
            Token::ForwardSlash,
            Constant(3),
            Token::Hyphen,
            Constant(4),
            Token::Asterisk,
            Constant(5),
        ];
        let res = parse_exp(&mut tokens.iter().peekable(), 0).unwrap();
        assert_eq!(
            res,
            Exp::Binary(
                BinaryOperator::Subtract,
                Box::new(Exp::Binary(
                    BinaryOperator::Add,
                    Box::new(Exp::Constant(1)),
                    Box::new(Exp::Binary(
                        BinaryOperator::Divide,
                        Box::new(Exp::Constant(2)),
                        Box::new(Exp::Constant(3))
                    ))
                )),
                Box::new(Exp::Binary(
                    BinaryOperator::Multiply,
                    Box::new(Exp::Constant(4)),
                    Box::new(Exp::Constant(5))
                ))
            )
        )
    }

    #[test]
    fn valid_binary_expression() {
        let tokens = vec![Constant(1), Token::Hyphen, Constant(2)];
        let res = parse_exp(&mut tokens.iter().peekable(), 0).unwrap();
        assert_eq!(
            res,
            Exp::Binary(
                crate::ast::BinaryOperator::Subtract,
                Box::new(Exp::Constant(1)),
                Box::new(Exp::Constant(2)),
            )
        );
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
        let res = parse_factor(&mut tokens.iter().peekable()).unwrap();
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
        if let Err(err) = parse_function(&mut tokens.iter().peekable()) {
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

        if let Err(err) = parse_function(&mut tokens.iter().peekable()) {
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

        if let Err(err) = parse_function(&mut tokens.iter().peekable()) {
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

        if let Err(err) = parse_function(&mut tokens.iter().peekable()) {
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

        if let Err(err) = parse_function(&mut tokens.iter().peekable()) {
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

        if let Err(err) = parse_function(&mut tokens.iter().peekable()) {
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

        if let Err(err) = parse_function(&mut tokens.iter().peekable()) {
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

        if let Err(err) = parse_function(&mut tokens.iter().peekable()) {
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

        let function_definition = parse_function(&mut tokens.iter().peekable()).unwrap();
        assert_eq!(
            function_definition,
            FunctionDefinition::Function("thing".to_string(), Statement::Return(Exp::Constant(4)))
        );
    }
}
