use alloc::string::String;

pub enum Token {
    Word(String),
    String(String),

    


    Eof,
}