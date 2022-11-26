use super::chunk::Chunk;

pub struct Function {
    pub chunk: Chunk,
    pub arity: u8,
    pub name: String,
}

impl Function {
    pub fn new() -> Function {
        Function {
            chunk: Chunk::new(),
            arity: 0,
            name: String::from("main"),
        }
    }
}
