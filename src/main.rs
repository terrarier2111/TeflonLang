use std::{env, fs};
use std::fs::File;
use std::io::{Error, ErrorKind, Write};

mod lexer;
mod parser;
mod diagnostics;

fn main() {
    let path = env::current_dir().unwrap();
    println!("The current directory is {}", path.display());
    let mut input = input("Please insert a path to a source file: ".to_owned()).unwrap();
    let file = fs::read_to_string(input).unwrap();
    println!("pre-lex");
    let lexed = lexer::lex(file).unwrap();
    println!("lexed!");
    println!("{:?}", lexed);
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

