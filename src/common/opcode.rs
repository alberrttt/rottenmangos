use strum::Display;

pub type ConstantLocation = u16;
pub type SlotIndex = u8;
pub type Offset = i16;
#[derive(Debug, Display, Clone)]
pub enum OpCode {
    Constant(ConstantLocation),
    // use string interning ? maybe
    // u16 is the location in the constant pool
    GetLocal(u16),
    DefineLocal(ConstantLocation),
    SetLocal(u16),

    DefineGlobal(ConstantLocation),
    GetGlobal(ConstantLocation),
    SetGlobal(ConstantLocation),
    TakeTempSlot(SlotIndex),
    SetTempSlot(SlotIndex),
    JumpIfFalse(Offset),
    Jump(Offset),
    Greater,
    Less,
    GreaterEq,
    LessEq,
    AssertEq,
    AssertNe,
    True,
    False,
    Void,
    Not,
    Negate,
    Pop,
    Print,
    Add,
    Sub,
    Div,
    Mul,
    Return,
    Nop,
}
