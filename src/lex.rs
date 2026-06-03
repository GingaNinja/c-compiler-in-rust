use regex::Regex;
use std::fmt::Display;
use strum::EnumProperty;

use crate::DccError;

#[derive(Debug, PartialEq, Clone, EnumProperty)]
pub enum Keyword {
    #[strum(props(Str = "int"))]
    Int = 0,
    #[strum(props(Str = "void"))]
    Void = 1,
    #[strum(props(Str = "return"))]
    Return = 2,
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display = if let Some(display) = self.get_str("Str") {
            display.to_string()
        } else {
            format!("{:?}", self)
        };
        write!(f, "{display}")
    }
}

#[derive(Debug, PartialEq, EnumProperty, Clone)]
pub enum Token {
    Keyword(Keyword),
    Identifier(String),
    Constant(i32),
    #[strum(props(Str = "("))]
    OpenParenthesis,
    #[strum(props(Str = ")"))]
    CloseParenthesis,
    #[strum(props(Str = "{"))]
    OpenBrace,
    #[strum(props(Str = "}"))]
    CloseBrace,
    #[strum(props(Str = ";"))]
    SemiColon,
    #[strum(props(Str = "~"))]
    Tilde,
    #[strum(props(Str = "-"))]
    Hyphen,
    #[strum(props(Str = "--"))]
    TwoHyphens,
    #[strum(props(Str = "+"))]
    Plus,
    #[strum(props(Str = "*"))]
    Asterisk,
    #[strum(props(Str = "/"))]
    ForwardSlash,
    #[strum(props(Str = "%"))]
    Percent,
    #[strum(props(Str = "&"))]
    Ampersand,
    #[strum(props(Str = "|"))]
    Pipe,
    #[strum(props(Str = "^"))]
    Hat,
    #[strum(props(Str = "<<"))]
    DoubleLeft,
    #[strum(props(Str = ">>"))]
    DoubleRight,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Keyword(keyword) => write!(f, "Keyword({})", keyword),
            Token::Identifier(ident) => write!(f, "Identifier({})", ident),
            _ => {
                let display = if let Some(display) = self.get_str("Str") {
                    display.to_string()
                } else {
                    format!("{:?}", self)
                };
                write!(f, "{display}")
            }
        }
    }
}

pub fn lex_source(input: &str) -> Result<Vec<Token>, DccError> {
    let keyword_map = vec![
        ("int", Keyword::Int),
        ("void", Keyword::Void),
        ("return", Keyword::Return),
    ];
    let token_map: Vec<(Regex, Box<dyn Fn(&str) -> Token>)> = vec![
        (
            Regex::new(r"^[a-z]\w*\b").unwrap(),
            Box::new(|found| Token::Identifier(found.to_owned())),
        ),
        (
            Regex::new(r"^[0-9]+\b").unwrap(),
            Box::new(|found| Token::Constant(found.parse().unwrap())),
        ),
        (
            Regex::new(r"^\(").unwrap(),
            Box::new(|_| Token::OpenParenthesis),
        ),
        (
            Regex::new(r"^\)").unwrap(),
            Box::new(|_| Token::CloseParenthesis),
        ),
        (Regex::new(r"^\+").unwrap(), Box::new(|_| Token::Plus)),
        (
            Regex::new(r"^\/").unwrap(),
            Box::new(|_| Token::ForwardSlash),
        ),
        (Regex::new(r"^\*").unwrap(), Box::new(|_| Token::Asterisk)),
        (Regex::new(r"^%").unwrap(), Box::new(|_| Token::Percent)),
        (Regex::new(r"^\{").unwrap(), Box::new(|_| Token::OpenBrace)),
        (Regex::new(r"^\}").unwrap(), Box::new(|_| Token::CloseBrace)),
        (Regex::new(r"^;").unwrap(), Box::new(|_| Token::SemiColon)),
        (Regex::new(r"^~").unwrap(), Box::new(|_| Token::Tilde)),
        (Regex::new(r"^-").unwrap(), Box::new(|_| Token::Hyphen)),
        (Regex::new(r"^--").unwrap(), Box::new(|_| Token::TwoHyphens)),
        (Regex::new(r"^\^").unwrap(), Box::new(|_| Token::Hat)),
        (Regex::new(r"^\|").unwrap(), Box::new(|_| Token::Pipe)),
        (Regex::new(r"^&").unwrap(), Box::new(|_| Token::Ampersand)),
        (Regex::new(r"^<<").unwrap(), Box::new(|_| Token::DoubleLeft)),
        (
            Regex::new(r"^>>").unwrap(),
            Box::new(|_| Token::DoubleRight),
        ),
    ];
    let mut pos = 1;
    let mut tokens = vec![];
    let catch_invalid = Regex::new(r"^[^\s]+\s").unwrap();
    let cur_length = input.len();
    let mut input = input.trim_start();
    if input.len() != cur_length {
        pos += cur_length - input.len();
    }

    while !input.is_empty() {
        let mut cur_found: Option<regex::Match> = None;
        let mut cur_found_token = None;
        for (re, get_enum) in &token_map {
            if let Some(found) = re.find(input) {
                let mut found_keyword = false;

                if cur_found.is_none() || found.as_str().len() > cur_found.unwrap().as_str().len() {
                    cur_found = Some(found);

                    let token = get_enum(found.as_str());
                    if let Token::Identifier(_) = token {
                        for (keyword_string, keyword) in keyword_map.iter() {
                            if found.as_str() == *keyword_string {
                                cur_found_token = Some(Token::Keyword(keyword.clone()));
                                found_keyword = true;
                                break;
                            }
                        }
                    }
                    if !found_keyword {
                        cur_found_token = Some(token);
                    }
                }
            }
        }
        if let Some(found) = cur_found {
            tokens.push(cur_found_token.unwrap());
            input = &input[found.end()..];
            pos += found.end();
        } else {
            if let Some(found) = catch_invalid.find(input) {
                return Err(DccError::InvalidInputChar {
                    pos,
                    found_char: found.as_str().trim_end().to_string(),
                });
            } else {
                return Err(DccError::InvalidInputChar {
                    pos,
                    found_char: input[0..].into(),
                });
            }
        }

        let cur_length = input.len();
        input = input.trim_start();
        if input.len() != cur_length {
            pos += cur_length - input.len();
        }
    }

    Ok(tokens)
}
#[cfg(test)]
mod test {
    use crate::lex::{
        Keyword::Int, Keyword::Return, Keyword::Void, Token::Ampersand, Token::Asterisk,
        Token::CloseBrace, Token::CloseParenthesis, Token::Constant, Token::DoubleLeft,
        Token::DoubleRight, Token::ForwardSlash, Token::Hat, Token::Identifier, Token::Keyword,
        Token::OpenBrace, Token::OpenParenthesis, Token::Percent, Token::Pipe, Token::Plus,
        Token::SemiColon, Token::TwoHyphens, lex_source,
    };

    #[test]
    fn keyword() {
        let input = "int";
        let tokens = lex_source(&input).expect("int should not error");
        assert_eq!(tokens, vec![Keyword(Int)]);
    }

    #[test]
    fn two_keywords() {
        let input = "int int";
        let tokens = lex_source(&input).expect("int should not error");
        assert_eq!(tokens, vec![Keyword(Int), Keyword(Int)]);
    }

    #[test]
    fn int_main() {
        let input = "int main";
        let tokens = lex_source(&input).unwrap();
        assert_eq!(tokens, vec![Keyword(Int), Identifier("main".to_string())]);
    }

    #[test]
    fn int_void_return() {
        let input = "int void return";
        let tokens = lex_source(&input).unwrap();
        assert_eq!(tokens, vec![Keyword(Int), Keyword(Void), Keyword(Return)]);
    }

    #[test]
    fn initial_space() {
        let input = " \n\tint";
        let tokens = lex_source(&input).expect("int should not error");
        assert_eq!(tokens, vec![Keyword(Int)]);
    }

    #[test]
    fn invalid_char() {
        let input = "main ! something else";
        match lex_source(&input) {
            Err(err) => assert_eq!(err.to_string(), "invalid input, char 6 - !"),
            Ok(_) => panic!("expecting failure here"),
        }
    }

    #[test]
    fn blank_doc() {
        let input = " ";
        let tokens = lex_source(&input).unwrap();
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn constant() {
        let input = "2";
        let tokens = lex_source(&input).unwrap();
        assert_eq!(tokens, vec![Constant(2)]);
    }

    #[test]
    fn plus_forwardslash_asterisk_percent() {
        let input = "+/*%";
        let tokens = lex_source(&input).unwrap();
        assert_eq!(tokens, vec![Plus, ForwardSlash, Asterisk, Percent]);
    }

    #[test]
    fn important_symbols() {
        let input = "(){};";

        let tokens = lex_source(&input).unwrap();
        assert_eq!(
            tokens,
            vec![
                OpenParenthesis,
                CloseParenthesis,
                OpenBrace,
                CloseBrace,
                SemiColon
            ]
        );
    }
    #[test]
    fn two_hyphens() {
        let input = "--";

        let tokens = lex_source(&input).unwrap();
        assert_eq!(tokens, vec![TwoHyphens])
    }

    #[test]
    fn bitwise() {
        let input = "^|&<<>>";

        let tokens = lex_source(&input).unwrap();
        assert_eq!(tokens, vec![Hat, Pipe, Ampersand, DoubleLeft, DoubleRight]);
    }
}
