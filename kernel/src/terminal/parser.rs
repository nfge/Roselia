use alloc::{string::ToString, vec::Vec};

use crate::{kprintln, terminal::{command::Command, token::Token}};

pub struct Parser;

impl Parser {
    pub fn parse(tokens: Vec<Token>) -> Option<Command> {
        let mut t_iter = tokens.into_iter();
        let mut args = Vec::new();
        let name = match t_iter.next()? {
            Token::Word(name) => name,
            _ => return None
        };
        for token in t_iter {
            match token {
                Token::Word(arg) | Token::String(arg) => args.push(arg),
                _ => break
            }
        }
        Some(Command::new(name, args))
    }
}