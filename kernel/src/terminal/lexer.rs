use alloc::{string::String, vec::Vec};

use super::token::Token;

pub struct Lexer;

impl Lexer {
    pub fn tokenize(line: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = line.chars().peekable();
        let mut current = String::new();
        let mut is_string = false;
        while let Some(c) = chars.next() {
            match c {
                ' ' if !is_string => {
                    if !current.is_empty() {
                        tokens.push(Token::Word(current.clone()));
                        current.clear();
                    }
                },
                '\\' if is_string => match chars.next() {
                    Some('n') => current.push('\n'),
                    Some('t') => current.push('\t'),
                    Some('r') => current.push('\r'),
                    Some('\\') => current.push('\\'),
                    Some(other) => {
                        current.push('\\');
                        current.push(other);
                    }
                    None => current.push('\\'),
                },
                '"' => {
                    if is_string {
                        if !current.is_empty() {
                            tokens.push(Token::String(current.clone()));
                            current.clear();
                        }
                        is_string = false;
                    } else {
                        is_string = true;
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }
        if !current.is_empty() {
            tokens.push(Token::Word(current));
        }
        tokens
    }
}
