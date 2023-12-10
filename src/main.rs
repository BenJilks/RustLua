use std::env::args;
use std::fs::File;
use std::io::Read;
use std::error::Error;

use lalrpop_util::lalrpop_mod;
use crate::interpreter::{Interpreter, Value};

lalrpop_mod!(pub lua_parser);

mod ast;
mod interpreter;

#[cfg(test)]
mod test;

fn execute_script(script: &str) -> interpreter::Result<Value> {
    let mut interpreter = Interpreter::new();
    interpreter.define("print", |arguments| {
        for (i, argument) in arguments.iter().enumerate() {
            if i == arguments.len() - 1 {
                println!("{}", argument);
            } else {
                print!("{} ", argument);
            }
        }
        Value::Nil
    });

    interpreter.execute(&script)
}

fn main() -> Result<(), Box<dyn Error>> {
    if args().len() < 2 {
        return Err(Box::from("Error: Please specify a lua script file to execute"));
    }

    for file_path in args().skip(1) {
        let mut file = File::open(file_path)?;
        let mut script = String::new();
        file.read_to_string(&mut script)?;

        execute_script(&script)?;
    }

    Ok(())
}
