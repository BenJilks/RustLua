use crate::interpreter::{Interpreter, Value, LuaError, self};

fn run_test_script(script: &str) -> interpreter::Result<Value> {
    let mut interpreter = Interpreter::new();
    interpreter.execute(script)
}

fn test_literal(lua: &str, expected: Value) {
    let x = run_test_script(&format!("return {}", lua))
        .expect("No errors");
    assert_eq!(x, expected);
}

#[test]
fn test_literals() {
    // Strings
    test_literal("\"Hello, World!\"", Value::String("Hello, World!".to_owned()));

    // Numbers
    test_literal("21", Value::Number(21.0));
    test_literal("21.5", Value::Number(21.5));
    test_literal(".5", Value::Number(0.5));
    test_literal("5.", Value::Number(5.));

    // Booleans
    test_literal("true", Value::Boolean(true));
    test_literal("false", Value::Boolean(false));

    // Table
    test_literal("{}", Value::Table(Default::default()));
}

#[test]
fn test_index_error() {
    assert_eq!(run_test_script("true.x"), Err(LuaError::InvalidIndex(Value::Boolean(true))));
}

#[test]
fn test_arithmetic_error() {
    assert_eq!(run_test_script("true + 1"), Err(LuaError::InvalidArithmetic(Value::Boolean(true))));
}

#[test]
fn test_call_error() {
    assert_eq!(run_test_script("true()"), Err(LuaError::InvalidCall(Value::Boolean(true))));
}
