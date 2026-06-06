use crate::ast::{self};

#[derive(Debug)]
pub struct Program {
    pub function: Function,
}

#[derive(Debug)]
pub struct Function {
    pub identifier: String,
    pub body: Vec<Instruction>,
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Return(Val),
    Unary {
        operator: UnaryOperator,
        src: Val,
        dst: Val,
    },
    Binary {
        operator: BinaryOperator,
        src1: Val,
        src2: Val,
        dst: Val,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Val {
    Constant(i32),
    Var(String),
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
    ShiftLeft,
    ShiftRight,
    BitwiseOr,
    BitwiseAnd,
    BitwiseXor,
}

impl From<ast::UnaryOperator> for UnaryOperator {
    fn from(value: ast::UnaryOperator) -> Self {
        match value {
            ast::UnaryOperator::Complement => UnaryOperator::Complement,
            ast::UnaryOperator::Negate => UnaryOperator::Negate,
            _ => todo!(),
        }
    }
}

impl From<ast::BinaryOperator> for BinaryOperator {
    fn from(value: ast::BinaryOperator) -> Self {
        match value {
            ast::BinaryOperator::Add => BinaryOperator::Add,
            ast::BinaryOperator::Subtract => BinaryOperator::Subtract,
            ast::BinaryOperator::Multiply => BinaryOperator::Multiply,
            ast::BinaryOperator::Divide => BinaryOperator::Divide,
            ast::BinaryOperator::Remainder => BinaryOperator::Remainder,
            ast::BinaryOperator::BitwiseShiftLeft => BinaryOperator::ShiftLeft,
            ast::BinaryOperator::BitwiseShiftRight => BinaryOperator::ShiftRight,

            ast::BinaryOperator::BitwiseAnd => BinaryOperator::BitwiseAnd,
            ast::BinaryOperator::BitwiseXOR => BinaryOperator::BitwiseXor,
            ast::BinaryOperator::BitwiseOr => BinaryOperator::BitwiseOr,
            _ => todo!(),
        }
    }
}

impl From<ast::Program> for Program {
    fn from(value: ast::Program) -> Self {
        Self {
            function: value.function.into(),
        }
    }
}

impl From<ast::FunctionDefinition> for Function {
    fn from(value: ast::FunctionDefinition) -> Self {
        match value {
            ast::FunctionDefinition::Function(name, statement) => Self {
                identifier: name,
                body: statement.into(),
            },
        }
    }
}

impl From<ast::Statement> for Vec<Instruction> {
    fn from(value: ast::Statement) -> Self {
        match value {
            ast::Statement::Return(exp) => {
                let mut instructions = vec![];
                let mut var_count = 0;
                let var = emit_tacky(exp, &mut instructions, &mut var_count);
                instructions.push(Instruction::Return(var));
                instructions
            }
        }
    }
}

fn emit_tacky(exp: ast::Exp, instructions: &mut Vec<Instruction>, var_count: &mut i32) -> Val {
    match exp {
        ast::Exp::Constant(num) => Val::Constant(num),
        ast::Exp::Unary(op, exp) => {
            let src = emit_tacky(*exp, instructions, var_count);
            let dst_name = format!("tmp.{}", var_count);
            *var_count += 1;
            let dst = Val::Var(dst_name);
            instructions.push(Instruction::Unary {
                operator: op.into(),
                src,
                dst: dst.clone(),
            });
            dst
        }
        ast::Exp::Binary(op, e1, e2) => {
            let v1 = emit_tacky(*e1, instructions, var_count);
            let v2 = emit_tacky(*e2, instructions, var_count);
            let dst_name = format!("tmp.{}", var_count);
            *var_count += 1;
            let dst = Val::Var(dst_name);
            instructions.push(Instruction::Binary {
                operator: op.into(),
                src1: v1,
                src2: v2,
                dst: dst.clone(),
            });
            dst
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{ast, tacky::Instruction};

    #[test]
    fn constant() {
        let ast = ast::Statement::Return(ast::Exp::Constant(2));
        let instructions: Vec<Instruction> = ast.into();
        assert_eq!(
            instructions,
            vec![Instruction::Return(crate::tacky::Val::Constant(2))]
        );
    }

    #[test]
    fn unary_constant() {
        let ast = ast::Statement::Return(ast::Exp::Unary(
            ast::UnaryOperator::Negate,
            Box::new(ast::Exp::Constant(3)),
        ));
        let instructions: Vec<Instruction> = ast.into();
        assert_eq!(
            instructions,
            vec![
                Instruction::Unary {
                    operator: crate::tacky::UnaryOperator::Negate,
                    src: crate::tacky::Val::Constant(3),
                    dst: crate::tacky::Val::Var("tmp.0".to_string())
                },
                Instruction::Return(crate::tacky::Val::Var("tmp.0".to_string()))
            ]
        );
    }

    #[test]
    fn nested_unary_constant() {
        let ast = ast::Statement::Return(ast::Exp::Unary(
            ast::UnaryOperator::Negate,
            Box::new(ast::Exp::Unary(
                ast::UnaryOperator::Complement,
                Box::new(ast::Exp::Unary(
                    ast::UnaryOperator::Negate,
                    Box::new(ast::Exp::Constant(8)),
                )),
            )),
        ));
        let instructions: Vec<Instruction> = ast.into();
        assert_eq!(
            instructions,
            vec![
                Instruction::Unary {
                    operator: crate::tacky::UnaryOperator::Negate,
                    src: crate::tacky::Val::Constant(8),
                    dst: crate::tacky::Val::Var("tmp.0".to_string())
                },
                Instruction::Unary {
                    operator: crate::tacky::UnaryOperator::Complement,
                    src: crate::tacky::Val::Var("tmp.0".to_string()),
                    dst: crate::tacky::Val::Var("tmp.1".to_string())
                },
                Instruction::Unary {
                    operator: crate::tacky::UnaryOperator::Negate,
                    src: crate::tacky::Val::Var("tmp.1".to_string()),
                    dst: crate::tacky::Val::Var("tmp.2".to_string())
                },
                Instruction::Return(crate::tacky::Val::Var("tmp.2".to_string()))
            ]
        )
    }
}
