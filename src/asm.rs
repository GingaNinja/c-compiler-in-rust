use std::fmt::Display;

#[derive(Debug)]
pub struct Program {
    pub function: FunctionDefinition,
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Program(\n    Function(\n        {}\n    )\n)",
            self.function
        )
    }
}

impl Program {
    pub fn asm(&self) -> Result<String, String> {
        let output = self.function.asm()?;

        Ok(output)
    }
}

#[derive(Debug, PartialEq)]
pub enum FunctionDefinition {
    Function(String, Vec<Instruction>),
}

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function(name, instructions) => {
                write!(f, "name=\"{name}\",\n        body={instructions:?}")
            }
        }
    }
}

impl FunctionDefinition {
    pub fn asm(&self) -> Result<String, String> {
        let output = match self {
            FunctionDefinition::Function(name, instructions) => {
                let listing: Vec<_> = instructions.iter().map(|i| i.asm()).collect();
                format!("\t.globl _{name}\n_{name}:\n{}", listing.join("\n"))
            }
        };

        Ok(output)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Instruction {
    Mov { source: Operand, dest: Operand },
    Ret,
}

impl Instruction {
    pub fn asm(&self) -> String {
        match self {
            Instruction::Ret => "\tret".to_string(),
            Instruction::Mov {
                source: left,
                dest: right,
            } => format!("\tmovl {}, {}", left.asm(), right.asm()),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Operand {
    Imm(i32),
    Register,
}

impl Operand {
    pub fn asm(&self) -> String {
        match self {
            Operand::Register => "%eax".to_string(),
            Operand::Imm(num) => format!("${}", num),
        }
    }
}
