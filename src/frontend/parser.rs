use std::{mem::transmute, ops::Range};

use colored::Colorize;

use crate::cli_context::Context;

use super::{
    ast::{
        declaration::variable_declaration::VariableDeclaration,
        expression::{
            block::Block, variable_assignment::VariableAssignment, AsExpr, BinaryExpr, Expression,
        },
        identifier::Identifier,
        literal::Literal,
        node::{AsNode, Node},
        statement::Statement,
        BinaryOperation,
    },
    file::FileNode,
    scanner::{Position, Scanner, Token, TokenKind},
    Precedence,
};

#[derive(Debug)]
pub struct Parser<'a> {
    pub tokens: Vec<Token>,
    pub index: usize,
    pub context: Option<&'a mut Context<'a>>,
    pub had_error: bool,
    pub scanner: Option<Box<Scanner>>,
    pub panic_mode: bool,
    pub scope_depth: usize,
}
pub struct Rule<'a> {
    pub precedence: Precedence,
    pub prefix: Option<fn(&mut Parser<'a>, can_assign: bool) -> Node>,
    pub infix: Option<fn(&mut Parser<'a>, previous: Node) -> Node>,
}

impl<'a> Parser<'a> {
    pub fn get_rule(kind: TokenKind) -> Rule<'a> {
        match kind {
            TokenKind::LeftBrace => Rule {
                precedence: Precedence::None,
                prefix: Some(|parser, can_assign| {
                    parser.begin_scope();
                    let mut block = Block {
                        declarations: Vec::new(),
                    };
                    loop {
                        if !parser.check(TokenKind::RightBrace) && !parser.check(TokenKind::EOF) {
                            block.declarations.push(parser.node())
                        } else {
                            break;
                        }
                    }
                    parser.consume(TokenKind::RightBrace, "Expected '}' after block to close");
                    parser.end_scope();
                    return block.as_node();
                }),
                infix: None,
            },
            TokenKind::Identifier => Rule {
                precedence: Precedence::None,
                prefix: Some(|parser, can_assign| {
                    let token = parser.previous().clone();
                    if can_assign && parser.match_token(TokenKind::Equal) {
                        return Expression::VariableAssignment(VariableAssignment {
                            name: Identifier { name: token },
                            initializer: Box::new(parser.expression().as_expr()),
                        })
                        .as_node();
                    }
                    Identifier { name: token }.as_node()
                }),
                infix: None,
            },
            TokenKind::Number => Rule {
                precedence: Precedence::None,
                infix: None,
                prefix: Some(Self::number),
            },
            TokenKind::Star | TokenKind::Slash => Rule {
                precedence: Precedence::Factor,
                prefix: None,
                infix: Some(Self::binary),
            },
            TokenKind::Plus => Rule {
                infix: Some(Self::binary),
                prefix: None,
                precedence: Precedence::Term,
            },
            TokenKind::Dash => Rule {
                infix: Some(Self::binary),
                prefix: Some(|parser, can_assign| {
                    Expression::Negate(Box::new(parser.precedence(Precedence::Unary).as_expr()))
                        .as_node()
                }),
                precedence: Precedence::Term,
            },
            TokenKind::String => Rule {
                precedence: Precedence::None,
                prefix: Some(Self::string),

                infix: None,
            },
            TokenKind::Equal | TokenKind::SemiColon | TokenKind::Comma => Rule {
                precedence: Precedence::None,
                infix: None,
                prefix: None,
            },
            _ => Rule {
                precedence: Precedence::Unimpl,
                infix: None,
                prefix: None,
            },
        }
    }

    pub fn at_end(&mut self) -> bool {
        self.index + 1 >= self.tokens.len()
    }
    pub fn precedence(&mut self, prec: Precedence) -> Node {
        self.advance();
        let path = self.context.as_ref().unwrap().file_path.to_str().unwrap();
        let previous = self.previous();
        let rule = Self::get_rule(previous.kind);
        let can_assign: bool = prec <= Precedence::Assignment;
        #[allow(unused_assignments)]
        let mut expression: Node = Node::None;
        if rule.prefix.is_some() {
            expression = rule.prefix.unwrap()(self, can_assign);
        } else {
            panic!(
                "expected expression {}:{}:{}, got {}",
                path,
                previous.position.line.start + 1,
                previous.position.start.start,
                previous.kind
            );
        }

        loop {
            if self.at_end() {
                break expression;
            }
            let current = self.current();
            let current_rule = Self::get_rule(current.kind);
            if current_rule.precedence == Precedence::Unimpl && cfg!(debug_assertions) {
                println!(
                    "{} {}",
                    format!("Unimplemented rule:").bold().on_red().yellow(),
                    current.kind
                );
            }
            if prec >= current_rule.precedence {
                break expression;
            }

            self.advance();
            let previous = self.previous();

            match Self::get_rule(previous.kind).infix {
                None => {}
                Some(infix) => {
                    expression = infix(self, expression);
                }
            }
        }
    }
    pub fn expression(&mut self) -> Node {
        self.precedence(Precedence::None)
    }
    pub fn parse_file(&mut self) -> FileNode<'a> {
        let mut file = FileNode::default();
        loop {
            if self.at_end() {
                break;
            }
            file.nodes.push(self.node());
        }
        file
    }

    pub fn node(&mut self) -> Node {
        self.statement()
    }
    pub fn expression_statement(&mut self) -> Node {
        let expr = self.expression().as_expr();
        self.consume(TokenKind::SemiColon, "Expected ';' after expression");
        Statement::Expression(expr).as_node()
    }
    pub fn token_as_identifier(&mut self) -> Identifier {
        self.advance();
        Identifier {
            name: self.previous().clone(),
        }
    }
    pub fn statement(&mut self) -> Node {
        match self.current().kind {
            TokenKind::Print => {
                self.advance();
                let node = Statement::Print(Box::new(self.expression())).as_node();
                self.consume(TokenKind::SemiColon, "");
                node
            }
            TokenKind::AssertEq => {
                self.advance();
                let lhs = self.expression().as_expr();
                self.consume(TokenKind::Comma, "Expected comma to seperate lhs and rhs");
                let rhs = self.expression().as_expr();
                self.consume(TokenKind::SemiColon, "");

                let node = Statement::AssertEq(lhs, rhs);
                node.as_node()
            }
            TokenKind::Let => {
                self.advance();
                let identifier = self.token_as_identifier();

                self.consume(TokenKind::Equal, "Expected '=' after variable name");
                let initializer = self.expression().as_expr();
                self.consume(
                    TokenKind::SemiColon,
                    "Expected ';' after variable declaration",
                );

                VariableDeclaration {
                    intializer: initializer,
                    identifier,
                    is_global: if self.scope_depth > 0 { false } else { true },
                }
                .as_node()
            }
            _ => self.expression_statement(),
        }
    }
}
impl Parser<'_> {
    pub fn string(&mut self, _can_assign: bool) -> Node {
        Literal::String(self.previous().value.clone()).as_node()
    }
    pub fn binary(&mut self, lhs: Node) -> Node {
        let rule = Self::get_rule(self.previous().kind);
        let op = match self.previous().kind {
            TokenKind::Plus => BinaryOperation::Add,
            TokenKind::Dash => BinaryOperation::Subtract,
            TokenKind::Star => BinaryOperation::Multiply,
            TokenKind::Slash => BinaryOperation::Divide,
            _ => panic!(),
        };
        // the precedence is +1 so it'll compile it as the rhs
        let prec: Precedence = unsafe { transmute((rule.precedence as u8) + 1) };
        let rhs = self.precedence(prec);

        Expression::Binary(BinaryExpr {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            op,
        })
        .as_node()
    }
    pub fn number(&mut self, _can_assign: bool) -> Node {
        Literal::Number(self.previous().value.parse::<f64>().unwrap()).as_node()
    }
}
const EOF: &Token = &Token {
    kind: TokenKind::EOF,
    value: String::new(),
    line: 0,
    length: 0,
    position: Position {
        line: 0..0,
        start: 0..0,
    },
};
impl<'a> Parser<'a> {
    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }
    pub fn end_scope(&mut self) {
        self.scope_depth -= 1;
    }
    pub fn error(&mut self, msg: &str) {
        let previous = self.previous().to_owned();

        self.error_at(&previous, msg);
        self.had_error = true;
    }
    pub fn error_at(&mut self, token: &Token, msg: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        println!("{} ->", "Compiler".yellow(),);
        print!(
            "    {}",
            format!(
                "[test.miso:{}:{}]: ",
                token.line + 1,
                token.position.start.start + 1
            )
            .blue(),
        );
        match token.kind {
            TokenKind::EOF => {
                print!("Error at EOF");
            }

            _ => {
                let range: Range<usize> =
                    (token.position.start.start as usize)..(token.position.start.end as usize);
                print!(
                    "Error at '{}'",
                    self.scanner.as_ref().unwrap().source[range].to_string()
                );
            }
        }
        println!(", {}", msg.red());
    }
    pub fn new(scanner: Box<Scanner>, context: Option<&'a mut Context<'a>>) -> Parser<'a> {
        let parser = Parser {
            tokens: scanner.tokens.to_owned(),
            index: 0,

            context,
            scanner: Some(scanner),
            had_error: false,
            panic_mode: false,
            scope_depth: 0,
        };

        parser
    }
    pub fn match_token(&mut self, tk: TokenKind) -> bool {
        if !self.check(tk) {
            return false;
        };
        self.advance();
        return true;
    }
    pub fn check(&mut self, tk: TokenKind) -> bool {
        self.current().kind == tk
    }
    pub fn previous(&mut self) -> &Token {
        &self.tokens[self.index - 1]
    }
    pub fn current(&mut self) -> &Token {
        if self.index > self.tokens.len() - 1 {
            return EOF;
        } else {
            &self.tokens[self.index]
        }
    }

    pub fn advance(&mut self) -> &Token {
        self.index += 1;
        &self.tokens[self.index - 1]
    }
    pub fn consume(&mut self, kind: TokenKind, err: &str) -> &Token {
        let advance = self.advance();

        if advance.kind.eq(&kind) {
            return advance;
        } else {
            panic!("{}", err)
        }
    }

    pub fn peek(&mut self, distance: usize) -> &Token {
        &self.tokens[self.index - (distance)]
    }
}
