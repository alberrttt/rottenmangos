use proc_macro::{self, TokenStream};

use syn::Arm;

struct Arms {
    arms: Vec<Arm>,
}
// the ground work for adding a better "api" for manipulating opcodes
mod opcode;
use opcode::expand_opcode as _expand_opcode;
#[proc_macro_derive(ExpandOpCode)]
pub fn expand_opcode(input: TokenStream) -> TokenStream {
    _expand_opcode(input)
}

mod make_tests;
use make_tests::make_tests as _make_tests;
#[proc_macro]
pub fn make_tests(_item: TokenStream) -> TokenStream {
    _make_tests(_item)
}

mod lookup;
use lookup::lookup as _lookup;
#[proc_macro]
pub fn lookup(input: TokenStream) -> TokenStream {
    _lookup(input)
}
