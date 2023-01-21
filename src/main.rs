#![feature(map_try_insert)]
#![feature(box_into_inner)]

use crate::lexer::token::Token;
use crate::parser::parser::Parser;
use crate::parser::token_stream::TokenStream;
use std::fs::File;
use std::io::{Error, ErrorKind, Write};
use std::{env, fs};
use crate::diagnostics::builder::DiagnosticBuilder;
use crate::parser::ast::{ItemKind, StmtKind};
use crate::tyck::{DEFAULT_PATH, Ty, tyck_item};

mod diagnostics;
mod lexer;
mod parser;
mod traitsolver;
mod tyck;

fn main() {
    let path = env::current_dir().unwrap();
    println!("The current directory is {}", path.display());
    let mut input = input("Please insert a path to a source file: ".to_owned()).unwrap();
    let file = fs::read_to_string(input).unwrap();
    println!("pre-lex");
    let lexed = lexer::lex(file).unwrap();
    let tokens = lexed.len();
    println!("lexed!");
    println!("{:?}", lexed);
    let mut token_stream = TokenStream::new(lexed);
    let mut parser = Parser::new(token_stream);
    println!("parsing...");
    let krate = parser.parse_crate().unwrap();
    println!("parsed!");
    println!("ast: {:?}", krate);
    println!("tokens: {}", tokens);
    println!("items: {}", krate.items.len());
    let mut tyck_ctx = krate.build_ctx();
    let mut diagnostics = DiagnosticBuilder::new();
    for item in &*krate.items {
        tyck_item(&mut tyck_ctx, item);
    }
}

// https://hackernoon.com/lets-build-a-programming-language-2612349105c6
// https://medium.com/hackernoon/compilers-and-interpreters-3e354a2e41cf

fn input(text: String) -> std::io::Result<String> {
    print!("{}", text);
    std::io::stdout().flush()?; // because print! doesn't flush
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input)? == 0 {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "EOF while reading a line",
        ));
    }
    if input.ends_with('\n') {
        input.pop();
        if input.ends_with('\r') {
            input.pop();
        }
    }
    Ok(input)
}
