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
fn test_function_call() {
    let x = run_test_script(r#"
        function x(a, b)
            return a + b
        end

        return x(1, 2)
    "#);

    assert_eq!(x, Ok(Value::Number(3.0)));
}

#[test]
fn test_locals() {
    let x = run_test_script(r#"
        function x()
            local l = 21
            return l
        end

        x()
        return l
    "#);
    assert_eq!(x, Ok(Value::Nil));

    let x = run_test_script(r#"
        function x()
            local l = 21
            return l
        end

        return x()
    "#);
    assert_eq!(x, Ok(Value::Number(21.0)));
}

#[test]
fn test_captures() {
    let x = run_test_script(r#"
        function foo()
            local l = 0
            return function()
                l = l + 1
                return l
            end
        end

        x = foo()
        x()
        x()
        return x()
    "#);
    assert_eq!(x, Ok(Value::Number(3.0)));

    let x = run_test_script(r#"
        function foo()
            return function()
                l = l + 1
                return l
            end
            local l = 0
        end

        x = foo()
        x()
        x()
        return x()
    "#);
    assert_eq!(x, Err(LuaError::InvalidArithmetic(Value::Nil)));
}

#[test]
fn test_if() {
    assert_eq!(run_test_script("if true then return 1 end"), Ok(Value::Number(1.0)));
    assert_eq!(run_test_script("if false then return 1 end"), Ok(Value::Nil));
    assert_eq!(run_test_script("if nil then return 1 end"), Ok(Value::Nil));
    assert_eq!(run_test_script("if 1 then return 1 end"), Ok(Value::Number(1.0)));
    assert_eq!(run_test_script("if {} then return 1 end"), Ok(Value::Number(1.0)));
    assert_eq!(run_test_script("if \"test\" then return 1 end"), Ok(Value::Number(1.0)));

    assert_eq!(run_test_script(r#"
        if 1 == 2 then
            return 1
        elseif 1 == 3 then
            return 2
        elseif 1 == 1 then
            return 3
        else
            return 4
        end
    "#), Ok(Value::Number(3.0)));
}

#[test]
fn test_numeric_for() {
    let x = run_test_script(r"
        x = 0
        for i = 0, 10, 5 do
            x = x + i
        end

        return x
    ");

    assert_eq!(x, Ok(Value::Number(15.0)));

    assert_eq!(run_test_script("for i = nil, 0 do end"), Err(LuaError::BadForInitialValue(Value::Nil)));
    assert_eq!(run_test_script("for i = 0, nil do end"), Err(LuaError::BadForLimit(Value::Nil)));
    assert_eq!(run_test_script("for i = 0, 1, nil do end"), Err(LuaError::BadForStep(Value::Nil)));
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
