use lalrpop_util::lalrpop_mod;
use crate::interpreter::{Interpreter, Value};

lalrpop_mod!(pub lua_parser);

mod ast;
mod interpreter;

fn main() {
    // let test_program = r"
    //     function test(a, b)
    //         local x = 21
    //         return a + b * x
    //     end
    // ";

    let test_program = r#"
        function x()
            local test = 0

            return function()
                test = test + 1
                return test
            end
        end

        test = x()
        print(test())
        print(test())
        print(test())
        print(test())
    "#;

    let parser = lua_parser::ProgramParser::new();
    let program = parser.parse(test_program).unwrap();

    let mut interpreter = Interpreter::new();
    interpreter.define("print", |arguments| {
        for argument in arguments {
            match argument {
                Value::Nil => print!("<nil> "),
                Value::Number(n) => print!("{} ", n),
                Value::String(s) => print!("{} ", s),
                Value::Table(table) => print!("{:?} ", table.borrow()),
                Value::Function(_) => print!("<function> "),
                Value::NativeFunction(_) => print!("<native function> "),
            }
        }

        println!();
        Value::Nil
    });

    interpreter.execute(program);
}
