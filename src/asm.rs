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

fn check_for_invalid_operands(instructions: &mut Vec<Instruction>) {
    let mut i = 0;
    while i < instructions.len() {
        match instructions[i].clone() {
            Instruction::Mov { source, dest } => {
                if let Operand::Stack(_) = source {
                    if let Operand::Stack(_) = dest {
                        instructions[i] = Instruction::Mov {
                            source: source.clone(),
                            dest: Operand::Reg(Reg::R10),
                        };
                        instructions.insert(
                            i + 1,
                            Instruction::Mov {
                                source: Operand::Reg(Reg::R10),
                                dest: dest.clone(),
                            },
                        );
                    }
                }
            }
            Instruction::Binary { operator, op1, op2 }
                if operator == BinaryOperator::Add
                    || operator == BinaryOperator::Sub
                    || operator == BinaryOperator::Or
                    || operator == BinaryOperator::Xor
                    || operator == BinaryOperator::And =>
            {
                if let Operand::Stack(_) = op1 {
                    if let Operand::Stack(_) = op2 {
                        instructions[i] = Instruction::Mov {
                            source: op1.clone(),
                            dest: Operand::Reg(Reg::R10),
                        };
                        instructions.insert(
                            i + 1,
                            Instruction::Binary {
                                operator,
                                op1: Operand::Reg(Reg::R10),
                                op2: op2.clone(),
                            },
                        );
                    }
                }
            }
            Instruction::Binary { operator, op1, op2 }
                if operator == BinaryOperator::Shl || operator == BinaryOperator::Sar =>
            {
                if let Operand::Stack(_) = op1 {
                    instructions[i] = Instruction::Mov {
                        source: op1.clone(),
                        dest: Operand::Reg(Reg::CL),
                    };
                    instructions.insert(
                        i + 1,
                        Instruction::Binary {
                            operator,
                            op1: Operand::Reg(Reg::CL),
                            op2: op2.clone(),
                        },
                    );
                }
            }
            Instruction::Binary { operator, op1, op2 } if operator == BinaryOperator::Mult => {
                if let Operand::Stack(_) = op2 {
                    instructions[i] = Instruction::Mov {
                        source: op2.clone(),
                        dest: Operand::Reg(Reg::R11),
                    };
                    instructions.insert(
                        i + 1,
                        Instruction::Binary {
                            operator,
                            op1: op1,
                            op2: Operand::Reg(Reg::R11),
                        },
                    );
                    instructions.insert(
                        i + 2,
                        Instruction::Mov {
                            source: Operand::Reg(Reg::R11),
                            dest: op2.clone(),
                        },
                    );
                }
            }

            Instruction::Idiv(src) => {
                if let Operand::Imm(_) = src {
                    instructions[i] = Instruction::Mov {
                        source: src,
                        dest: Operand::Reg(Reg::R10),
                    };
                    instructions.insert(i + 1, Instruction::Idiv(Operand::Reg(Reg::R10)));
                }
            }
            _ => {}
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
                Instruction::Binary { operator, op1, op2 } => {
                    let op1 = replace_pseudo(op1, &mut pseudo_lookup, &mut cur_stack_loc);
                    let op2 = replace_pseudo(op2, &mut pseudo_lookup, &mut cur_stack_loc);
                    Instruction::Binary {
                        operator: operator.clone(),
                        op1,
                        op2,
                    }
                }
                Instruction::Idiv(src) => {
                    let src = replace_pseudo(src, &mut pseudo_lookup, &mut cur_stack_loc);
                    Instruction::Idiv(src)
                }
                _ => continue,
            };
            *i = new_i;
        }
        instructions.insert(0, Instruction::AllocateStack(cur_stack_loc.abs()));

        check_for_invalid_operands(&mut instructions);

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
    Binary {
        operator: BinaryOperator,
        op1: Operand,
        op2: Operand,
    },
    Idiv(Operand),
    Cdq,
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
            tacky::Instruction::Binary {
                operator,
                src1,
                src2,
                dst,
            } if *operator != tacky::BinaryOperator::Divide
                && *operator != tacky::BinaryOperator::Remainder =>
            {
                instructions.push(Self::Mov {
                    source: src1.into(),
                    dest: dst.into(),
                });
                instructions.push(Self::Binary {
                    operator: operator.into(),
                    op1: src2.into(),
                    op2: dst.into(),
                });
            }
            tacky::Instruction::Binary {
                operator,
                src1,
                src2,
                dst,
            } => {
                instructions.push(Self::Mov {
                    source: src1.into(),
                    dest: Operand::Reg(Reg::AX),
                });
                instructions.push(Self::Cdq);
                instructions.push(Self::Idiv(src2.into()));
                let output_reg = if *operator == tacky::BinaryOperator::Divide {
                    Reg::AX
                } else {
                    Reg::DX
                };
                instructions.push(Self::Mov {
                    source: Operand::Reg(output_reg),
                    dest: dst.into(),
                })
            }
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
            } => {
                if let Operand::Reg(Reg::CL) = right {
                    write!(f, "\tmovb\t{}, {}", left, right)
                } else {
                    write!(f, "\tmovl\t{}, {}", left, right)
                }
            }
            Instruction::Unary { operator, dest } => {
                write!(f, "\t{}\t{}", operator, dest)
            }
            Instruction::AllocateStack(int) => write!(f, "\tsubq\t${}, %rsp", int),
            Instruction::Binary { operator, op1, op2 } => {
                write!(f, "\t{operator}\t{op1},\t{op2}")
            }
            Instruction::Cdq => write!(f, "\tcdq"),
            Instruction::Idiv(operand) => write!(f, "\tidivl\t{operand}"),
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
pub enum BinaryOperator {
    Add,
    Sub,
    Mult,
    DivRem,
    Shl,
    Sar,
    Or,
    And,
    Xor,
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "addl"),
            BinaryOperator::Sub => write!(f, "subl"),
            BinaryOperator::Mult => write!(f, "imull"),
            BinaryOperator::Shl => write!(f, "shll"),
            BinaryOperator::Sar => write!(f, "sarl"),
            BinaryOperator::Or => write!(f, "orl"),
            BinaryOperator::And => write!(f, "andl"),
            BinaryOperator::Xor => write!(f, "xorl"),
            _ => write!(f, "xxxx"),
        }
    }
}

impl From<&tacky::BinaryOperator> for BinaryOperator {
    fn from(value: &tacky::BinaryOperator) -> Self {
        match value {
            tacky::BinaryOperator::Add => BinaryOperator::Add,
            tacky::BinaryOperator::Subtract => BinaryOperator::Sub,
            tacky::BinaryOperator::Multiply => BinaryOperator::Mult,
            tacky::BinaryOperator::ShiftLeft => BinaryOperator::Shl,
            tacky::BinaryOperator::ShiftRight => BinaryOperator::Sar,
            tacky::BinaryOperator::BitwiseAnd => BinaryOperator::And,
            tacky::BinaryOperator::BitwiseOr => BinaryOperator::Or,
            tacky::BinaryOperator::BitwiseXor => BinaryOperator::Xor,
            _ => BinaryOperator::DivRem,
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
    CL,
    DX,
    R10,
    R11,
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Reg(reg) => match reg {
                Reg::AX => write!(f, "%eax"),
                Reg::DX => write!(f, "%edx"),
                Reg::CL => write!(f, "%cl"),
                Reg::R10 => write!(f, "%r10d"),
                Reg::R11 => write!(f, "%r11d"),
            },
            Operand::Imm(num) => write!(f, "${}", num),
            Operand::Stack(num) => write!(f, "{}(%rbp)", num),
            Operand::Pseudo(num) => write!(f, "pseudo({})", num),
        }
    }
}
