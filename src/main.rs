use lalrpop_util::lalrpop_mod;
use crate::interpreter::{Interpreter, Value};

lalrpop_mod!(pub lua_parser);

mod ast;
mod interpreter;

#[cfg(test)]
mod test;

fn main() -> interpreter::Result<()> {
    let test_program = r#"
        if 1 == 2 then
            print(21)
        elseif 1 == 1 then
            print(22)
        else
            print(23)
        end
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
