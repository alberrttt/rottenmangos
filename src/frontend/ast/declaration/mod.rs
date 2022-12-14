use crate::frontend::compiler::Compiler;

use self::{function::FunctionDeclaration, variable_declaration::VariableDeclaration};

use super::{node::AsNode, CompileToBytecode};

pub mod function;
pub mod variable_declaration;
#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    VariableDeclaration(VariableDeclaration),
    FunctionDeclaration(FunctionDeclaration),
}

impl CompileToBytecode for Declaration {
    fn to_bytecode(&self, compiler: &mut Compiler) {
        let _function = &mut compiler.bytecode.function;
        match self {
            Declaration::VariableDeclaration(declaration) => declaration.to_bytecode(compiler),
            Declaration::FunctionDeclaration(function_declaration) => {
                function_declaration.to_bytecode(compiler)
            }
        }
    }
}
pub trait AsDeclaration {
    fn to_declaration(self) -> Declaration;
}
impl AsNode for Declaration {
    fn to_node(self) -> super::node::Node {
        super::node::Node::Declaration(self)
    }
}
