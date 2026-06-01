use std::{collections::HashMap, fmt::Display};

use crate::tacky;

#[derive(Debug)]
pub struct Program {
    pub function: FunctionDefinition,
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.function)
    }
}

impl From<tacky::Program> for Program {
    fn from(value: tacky::Program) -> Self {
        Self {
            function: value.function.into(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FunctionDefinition {
    Function(String, Vec<Instruction>),
}

fn check_for_invalid_movs(instructions: &mut Vec<Instruction>) {
    let mut i = 0;
    while i < instructions.len() {
        if let Instruction::Mov { source, dest } = instructions[i].clone() {
            if let Operand::Stack(source_stack) = source {
                if let Operand::Stack(dest_stack) = dest {
                    instructions[i] = Instruction::Mov {
                        source: Operand::Stack(source_stack.clone()),
                        dest: Operand::Reg(Reg::R10),
                    };
                    instructions.insert(
                        i + 1,
                        Instruction::Mov {
                            source: Operand::Reg(Reg::R10),
                            dest: Operand::Stack(dest_stack.clone()),
                        },
                    );
                }
            }
        }
        i += 1;
    }
}

fn replace_pseudo(
    input: &Operand,
    pseudo_lookup: &mut HashMap<String, i32>,
    cur_stack_loc: &mut i32,
) -> Operand {
    if let Operand::Pseudo(ident) = input {
        let ident = ident.clone();

        let stack_loc = {
            if let Some(stack_loc) = pseudo_lookup.get(&ident) {
                *stack_loc
            } else {
                *cur_stack_loc -= 4;
                pseudo_lookup.insert(ident, *cur_stack_loc);
                *cur_stack_loc
            }
        };
        Operand::Stack(stack_loc)
    } else {
        input.clone()
    }
}

impl From<tacky::Function> for FunctionDefinition {
    fn from(value: tacky::Function) -> Self {
        let mut instructions = vec![];
        for i in value.body {
            Instruction::insert_from(&i, &mut instructions);
        }

        let mut pseudo_lookup = HashMap::new();
        let mut cur_stack_loc = 0;
        for i in instructions.iter_mut() {
            let new_i = match i {
                Instruction::Mov { source, dest } => {
                    let source = replace_pseudo(&source, &mut pseudo_lookup, &mut cur_stack_loc);
                    let dest = replace_pseudo(&dest, &mut pseudo_lookup, &mut cur_stack_loc);
                    Instruction::Mov { source, dest }
                }
                Instruction::Unary { operator, dest } => {
                    let dest = replace_pseudo(&dest, &mut pseudo_lookup, &mut cur_stack_loc);
                    Instruction::Unary {
                        operator: operator.clone(),
                        dest,
                    }
                }
                _ => continue,
            };
            *i = new_i;
        }
        instructions.insert(0, Instruction::AllocateStack(cur_stack_loc.abs()));

        check_for_invalid_movs(&mut instructions);

        Self::Function(value.identifier, instructions)
    }
}

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function(name, instructions) => {
                let listing: Vec<String> = instructions.iter().map(|i| i.to_string()).collect();
                write!(
                    f,
                    "\t.globl _{name}\n_{name}:\n\tpushq\t%rbp\n\tmovq\t%rsp, %rbp\n{}",
                    listing.join("\n")
                )
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Instruction {
    Mov {
        source: Operand,
        dest: Operand,
    },
    Unary {
        operator: UnaryOperator,
        dest: Operand,
    },
    AllocateStack(i32),
    Ret,
}

impl Instruction {
    pub fn insert_from(instr: &tacky::Instruction, instructions: &mut Vec<Instruction>) {
        match &instr {
            tacky::Instruction::Return(val) => {
                instructions.push(Self::Mov {
                    source: val.into(),
                    dest: Operand::Reg(Reg::AX),
                });
                instructions.push(Self::Ret);
            }
            &tacky::Instruction::Unary { operator, src, dst } => {
                instructions.push(Self::Mov {
                    source: src.into(),
                    dest: dst.into(),
                });
                instructions.push(Self::Unary {
                    operator: operator.into(),
                    dest: dst.into(),
                });
            }
            _ => todo!(),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Ret => write!(f, "\tmovq\t%rbp, %rsp\n\tpopq\t%rbp\n\tret"),
            Instruction::Mov {
                source: left,
                dest: right,
            } => write!(f, "\tmovl\t{}, {}", left, right),
            Instruction::Unary { operator, dest } => {
                write!(f, "\t{}\t{}", operator, dest)
            }
            Instruction::AllocateStack(int) => write!(f, "\tsubq\t${}, %rsp", int),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UnaryOperator {
    Neg,
    Not,
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOperator::Neg => write!(f, "negl"),
            UnaryOperator::Not => write!(f, "notl"),
        }
    }
}

impl From<&tacky::UnaryOperator> for UnaryOperator {
    fn from(value: &tacky::UnaryOperator) -> Self {
        match value {
            tacky::UnaryOperator::Complement => UnaryOperator::Not,
            tacky::UnaryOperator::Negate => UnaryOperator::Neg,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Operand {
    Imm(i32),
    Reg(Reg),
    Pseudo(String),
    Stack(i32),
}

impl From<&tacky::Val> for Operand {
    fn from(value: &tacky::Val) -> Self {
        match value {
            tacky::Val::Constant(int) => Self::Imm(*int),
            tacky::Val::Var(ident) => Self::Pseudo(ident.clone()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Reg {
    AX,
    R10,
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Reg(reg) => match reg {
                Reg::AX => write!(f, "%eax"),
                Reg::R10 => write!(f, "%r10d"),
            },
            Operand::Imm(num) => write!(f, "${}", num),
            Operand::Stack(num) => write!(f, "{}(%rbp)", num),
            Operand::Pseudo(num) => write!(f, "pseudo({})", num),
        }
    }
}
