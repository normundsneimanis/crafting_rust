// https://craftinginterpreters.com/compiling-expressions.html
// https://github.com/tdp2110/crafting-interpreters-rs/blob/trunk/src/treewalk_interpreter.rs#L116
mod scanner;
mod token;
mod expr;
mod parser;
mod ast_printer;
mod interpreter;
mod environment;
mod stmt;
mod vm;

use std::io::{stdout, Write};
use clap::{command, arg};
use crate::vm::{Chunk, OpCode, SrcLocation, Vm};

fn run(prog: &String, has_error: &mut bool) {
    let mut chunk = Chunk::default();
    let constant = chunk.add_constant(1.2);
    chunk.write_chunk(OpCode::OpConstant as u8, SrcLocation{col: 11, line: 1});
    chunk.write_chunk(constant, SrcLocation{col: 12, line: 1});

    let constant = chunk.add_constant(3.4);
    chunk.write_chunk(OpCode::OpConstant as u8, SrcLocation{col: 13, line: 1});
    chunk.write_chunk(constant, SrcLocation{col: 14, line: 1});
    chunk.write_chunk(OpCode::OpAdd as u8, SrcLocation{col: 15, line: 1});

    let constant = chunk.add_constant(5.6);
    chunk.write_chunk(OpCode::OpConstant as u8, SrcLocation{col: 16, line: 1});
    chunk.write_chunk(constant, SrcLocation{col: 17, line: 1});

    chunk.write_chunk(OpCode::OpDivide as u8, SrcLocation{col: 33, line: 3});
    chunk.write_chunk(OpCode::OpNegate as u8, SrcLocation{col: 44, line: 4});
    chunk.write_chunk(OpCode::OpReturn as u8, SrcLocation{col: 55, line: 2});
    chunk.disassemble(&"test chunk");

    let mut vm = Vm::default();
    vm.enable_debug();
    vm.interpret(chunk);


    let mut scanner = scanner::Scanner::default();
    scanner.set_source(prog);
    let mut parser = parser::Parser::default();

    *has_error = scanner.had_error();

    let tokens = scanner.scan_tokens();
    for token in tokens.clone() {
        println!("Token: {}", token.to_string());
    }

    let expr = parser.parse(tokens);
    // match expr {
    //     Ok(res) => {println!("Parsing successful: {}", res.to_string())},
    //     Err(err) => println!("Parse error: {}", err.to_string()),
    // }

    let mut interpreter = interpreter::Interpreter::default();
    interpreter.interpret(expr);
    // let result = interpreter.interpret(expr);
    //
    // println!("{}", match result {
    //     Ok(v) => v.to_string(),
    //     Err(e) => e.to_string(),
    // })
}


fn run_file(name: &String) {
    if let Ok(contents) = String::from_utf8(std::fs::read(name).unwrap()) {
        let mut has_error: bool = false;
        run(&contents, &mut has_error);
        if has_error {
            std::process::exit(64);
        }
    } else {
        println!("Failed to convert to string from utf8");
    }
}


fn run_prompt() {
    let mut line: String = Default::default();
    let mut bytes: usize;
    let mut has_error: bool = false;

    loop {
        print!("> ");
        let _ = stdout().flush();
        bytes = std::io::stdin().read_line(&mut line).unwrap();
        if bytes == 0 {
            break;
        }
        run(&line, &mut has_error);
        if has_error {
            has_error = false;
        }
        line.clear();
    }
}


fn main() {
    let matches = command!()
        .arg(arg!([name] "Optional file name to process"))
        .get_matches();

    if let Some(n) = matches.get_one::<String>("name") {
        run_file(&n);
    } else {
        run_prompt();
    }
    std::process::exit(0);
}
