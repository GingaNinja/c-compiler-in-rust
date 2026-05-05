mod asm;
mod ast;

use std::{
    fmt::Display,
    fs::{self, File, remove_file},
    io::{self, Read, Write},
    path::Path,
    process::Command,
};

use clap::Parser;
use regex::Regex;
use strum::EnumProperty;

use crate::ast::parse_program;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Input source code (expecting a c file)
    input_file: String,

    /// Direct compiler to run the lexer, but stop before parsing
    #[arg(short, long, value_name = "lex")]
    lex: bool,

    /// Direct compiler to run the lexer and parser, but stop before assembly generation
    #[arg(short, long)]
    parse: bool,

    /// Direct compiler to perform lexing, parsing, and assembly generation, but stop before code emission
    #[arg(short, long)]
    codegen: bool,

    /// Show debugging
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    println!("{} is the input file", cli.input_file);

    let input_file = Path::new(&cli.input_file);

    let preprocessed = set_file_extension(&input_file, "i");
    println!("{} is the pre-processed file", preprocessed);

    preprocess(&cli.input_file, &preprocessed)?;

    let compiled = set_file_extension(&input_file, "s");
    compile(
        &preprocessed,
        &compiled,
        cli.lex,
        cli.parse,
        cli.codegen,
        cli.debug,
    )?;

    remove_file(&preprocessed).expect("issue deleting pre-processed file");

    if cli.lex || cli.parse || cli.codegen {
        return Ok(());
    }

    let exe = set_file_extension(&input_file, "");
    create_exe(&compiled, &exe)?;

    remove_file(&compiled).expect("issue deleting compiled file");
    Ok(())
}

fn create_exe(input_file: &str, output_file: &str) -> Result<(), String> {
    let output = Command::new("gcc")
        .args([input_file, "-o", output_file])
        .output()
        .expect("failed to execute gcc");
    println!("status of gcc: {}", output.status);
    io::stdout()
        .write_all(&output.stdout)
        .expect("error writing to stdout");
    io::stderr()
        .write_all(&output.stderr)
        .expect("error writing to stderr");

    if output.status.success() {
        Ok(())
    } else {
        Err("issue running pre-process".to_string())
    }
}

fn preprocess(input_file: &str, output_file: &str) -> Result<(), String> {
    let output = Command::new("gcc")
        .args(["-E", "-P", input_file, "-o", output_file])
        .output()
        .expect("failed to execute gcc");
    io::stdout()
        .write_all(&output.stdout)
        .expect("error writing to stdout");
    io::stderr()
        .write_all(&output.stderr)
        .expect("error writing to stderr");

    if output.status.success() {
        Ok(())
    } else {
        Err("issue running pre-process".to_string())
    }
}

#[derive(Debug, PartialEq, Clone, EnumProperty)]
enum Keyword {
    #[strum(props(Str = "int"))]
    Int,
    #[strum(props(Str = "void"))]
    Void,
    #[strum(props(Str = "return"))]
    Return,
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

#[derive(Debug, PartialEq, EnumProperty)]
enum Token {
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
        // if let Token::Keyword(keyword) = self {
        //     write!(f, "Keyword({})", keyword)
        // } else if let Token::Identifier(ident) = self {
        //     write!(f, "Identifier({})", ident)
        // } else {
        //     let display = if let Some(display) = self.get_str("Str") {
        //         display.to_string()
        //     } else {
        //         format!("{:?}", self)
        //     };
        //     write!(f, "{display}")
        // }
    }
}

fn lex_source(input: &str) -> Result<Vec<Token>, String> {
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
        (Regex::new(r"^\{").unwrap(), Box::new(|_| Token::OpenBrace)),
        (Regex::new(r"^\}").unwrap(), Box::new(|_| Token::CloseBrace)),
        (Regex::new(r"^;").unwrap(), Box::new(|_| Token::SemiColon)),
        (Regex::new(r"^~").unwrap(), Box::new(|_| Token::Tilde)),
        (Regex::new(r"^-").unwrap(), Box::new(|_| Token::Hyphen)),
        (Regex::new(r"^--").unwrap(), Box::new(|_| Token::TwoHyphens)),
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
            // if found_token {x
            tokens.push(cur_found_token.unwrap());
            input = &input[found.end()..];
            pos += found.end();
        } else {
            if let Some(found) = catch_invalid.find(input) {
                return Err(format!(
                    "invalid input, char {pos} - {}",
                    found.as_str().trim_end()
                ));
            } else {
                return Err(format!("invalid input, char {pos} - {}", &input[0..]));
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

fn compile(
    input_file: &str,
    output_file: &str,
    lex: bool,
    parse: bool,
    codegen: bool,
    debug: bool,
) -> Result<(), String> {
    let mut input_file = File::open(input_file).expect("unable to open input file");
    let mut input = String::new();
    input_file
        .read_to_string(&mut input)
        .expect("unable to read input file");
    let tokens = lex_source(&input)?;
    if lex {
        return Ok(());
    }

    let program = parse_program(&mut tokens.iter())?;
    if debug {
        println!("Program ast: \n{}", program);
    }
    let asm = program.to_asm();
    if parse || codegen {
        return Ok(());
    }

    let assembly = asm.asm()?;
    fs::write(&output_file, assembly).expect("couldn't output to file");
    Ok(())
}

fn set_file_extension(input_file: &Path, extension: &str) -> String {
    if extension.len() == 0 {
        format!(
            "{}/{}",
            input_file.parent().unwrap().to_str().unwrap(),
            input_file
                .file_prefix()
                .expect("issues...")
                .to_str()
                .unwrap(),
        )
    } else {
        format!(
            "{}/{}.{}",
            input_file.parent().unwrap().to_str().unwrap(),
            input_file
                .file_prefix()
                .expect("issues...")
                .to_str()
                .unwrap(),
            extension
        )
    }
}

#[cfg(test)]
mod test {
    use crate::{
        Keyword::Int, Keyword::Return, Keyword::Void, Token::CloseBrace, Token::CloseParenthesis,
        Token::Constant, Token::Identifier, Token::Keyword, Token::OpenBrace,
        Token::OpenParenthesis, Token::SemiColon, Token::TwoHyphens, lex_source,
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
            Err(msg) => assert_eq!(msg, "invalid input, char 6 - !"),
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
}
