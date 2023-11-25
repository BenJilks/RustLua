use lalrpop_util::lalrpop_mod;
use crate::interpreter::{Interpreter, Value};

lalrpop_mod!(pub lua_parser);

mod ast;
mod interpreter;

#[cfg(test)]
mod test;

fn main() -> interpreter::Result<()> {
    let test_program = r#"
        print({ a = 1, b = 2 })
        print({ "a", "b" })
        print({ [0] = "a", [4] = "b" })
    "#;

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

    interpreter.execute(test_program)?;
    Ok(())
}
