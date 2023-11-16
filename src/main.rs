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

    let test_program = r"
        function test(a, b)
            local x = 1
            return a + b + x
        end

        print(test(1, 2))
        print(x)
    ";

    let parser = lua_parser::ProgramParser::new();
    let program = parser.parse(test_program).unwrap();

    let mut interpreter = Interpreter::new();
    interpreter.define("print", |arguments| {
        for argument in arguments {
            match argument {
                Value::Number(n) => print!("{} ", n),
                Value::Nil => print!("<nil> "),
                Value::Function(_, _) => print!("<function> "),
                Value::NativeFunction(_) => print!("<native function> "),
            }
        }

        println!();
        Value::Nil
    });

    interpreter.execute(program);
}
