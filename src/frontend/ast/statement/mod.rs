use crate::{common::opcode::OpCode, frontend::compiler::Compiler};

use super::{
    expression::{AsExpr, Expression},
    node::{AsNode, Node},
    CompileToBytecode,
};
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Expression),
    Print(Box<Node>),
    AssertEq(Expression, Expression),
}

impl AsNode for Statement {
    fn as_node(self) -> super::node::Node {
        super::node::Node::Statement(self)
    }
}
impl AsExpr for Statement {
    fn as_expr(self) -> Expression {
        match self {
            Statement::Expression(expr) => expr,
            _ => panic!(),
        }
    }
}
impl CompileToBytecode for Statement {
    fn to_bytecode(self, compiler: &mut Compiler) -> () {
        match self {
            Statement::Expression(expr) => {
                expr.to_bytecode(compiler);
                compiler.function.chunk.emit_op(OpCode::Pop)
            }
            Statement::Print(expr) => {
                expr.to_bytecode(compiler);
                compiler.function.chunk.emit_op(OpCode::Print);
            }
            Statement::AssertEq(lhs, rhs) => {
                lhs.to_bytecode(compiler);
                rhs.to_bytecode(compiler);

                compiler.function.chunk.emit_op(OpCode::AssertEq)
            }
        }
    }
}

pub trait AsStatement {
    fn as_statement(self) -> Statement;
}