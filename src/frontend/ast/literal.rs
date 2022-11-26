use crate::common::value::{AsValue, Value};

use super::{node::Node, CompileToBytecode};

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
}

impl Literal {
    pub fn as_node(self) -> Node {
        Node::Literal(self)
    }
    pub fn as_number(self) -> f64 {
        match self {
            Literal::Number(number) => return number,
            Literal::String(_) => panic!(),
        }
    }
}

impl CompileToBytecode for Literal {
    fn to_bytecode(self, function: &mut crate::common::function::Function) -> () {
        match self {
            Literal::Number(number) => function.chunk.emit_value(Value::Number(number)),
            Literal::String(string) => function.chunk.emit_value(string.to_string().as_value()),
        }
    }
}
