mod asm;
mod ast;
mod lex;
mod tacky;

use std::{
    fs::{self, File, remove_file},
    io::{self, Read, Write},
    path::Path,
    process::Command,
};

use clap::Parser;

use crate::{ast::parse_program, lex::lex_source};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Input source code (expecting a c file)
    input_file: String,

    /// Direct compiler to run the lexer, but stop before parsing
    #[arg(short, long, value_name = "lex")]
    lex: bool,

    /// Direct compiler to run the lexer and parser, but stop before generation generation
    #[arg(short, long)]
    parse: bool,

    /// Direct compiler to perform lexing, parsing, tacky generation, and  assembly generation, but stop before code emission
    #[arg(short, long)]
    codegen: bool,

    /// Show debugging
    #[arg(short, long)]
    debug: bool,

    /// Direct compiler to perform lexing, parsing, and tacky generation, but stop before assembly generation
    #[arg(short, long)]
    tacky: bool,
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
        cli.tacky,
        cli.codegen,
        cli.debug,
    )?;

    remove_file(&preprocessed).expect("issue deleting pre-processed file");

    if cli.lex || cli.parse || cli.codegen || cli.tacky {
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

fn compile(
    input_file: &str,
    output_file: &str,
    lex: bool,
    parse: bool,
    tacky: bool,
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
    if parse {
        return Ok(());
    }
    let tacky_ast: tacky::Program = program.into();
    if tacky {
        return Ok(());
    }
    let asm: asm::Program = tacky_ast.into();
    if codegen {
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
