use crate::ast::Statement;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use core::fmt;

use super::error::LuaError;
use super::Result;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Index {
    Name(String),
    Number(i32),
}

type Table = HashMap<Index, Value>;

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCapture {
    pub parameters: Vec<String>,
    pub body: Vec<Statement>,
    pub capture: Scope,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Nil,
    Number(f64),
    String(String),
    Boolean(bool),
    Function(Rc<FunctionCapture>),
    Table(Rc<RefCell<Table>>),
    NativeFunction(fn(Vec<Value>) -> Value),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "<nil>"),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Table(table) => write!(f, "{:?}", table.borrow()),
            Value::Function(_) => write!(f, "<function>"),
            Value::NativeFunction(_) => write!(f, "<native function>"),
        }
    }
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Nil => "nil",
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Boolean(_) => "boolean",
            Self::Function(_) | Self::NativeFunction(_) => "function",
            Self::Table(_) => "table",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Boolean(false) | Self::Nil => false,
            _ => true,
        }
    }
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Scope {
    table: HashMap<String, Rc<RefCell<Value>>>,
}

impl Scope {
    pub fn put(&mut self, name: String, value: Value) {
        match self.table.get(&name) {
            Some(slot) => slot.swap(&RefCell::from(value)),
            None => { self.table.insert(name, Rc::from(RefCell::from(value))); },
        }
    }

    pub fn has(&self, name: &str) -> bool {
        self.table.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.table.get(name).map(|x| x.borrow().clone())
    }
}

pub fn execute_arithmetic_operation(lhs: Value,
                                    rhs: Value,
                                    number_operation: fn(f64, f64) -> f64) -> Result<Value> {
    match lhs {
        Value::Nil => Err(LuaError::InvalidArithmetic(lhs)),
        Value::Number(lhs_n) => match rhs {
            Value::Nil => Err(LuaError::InvalidArithmetic(rhs)),
            Value::Number(rhs_n) => Ok(Value::Number(number_operation(lhs_n, rhs_n))),
            Value::String(_) => todo!("Implement string, number operations"),
            Value::Boolean(_) => Err(LuaError::InvalidArithmetic(rhs)),
            Value::Table(_) => Err(LuaError::InvalidArithmetic(rhs)),
            Value::Function(_) => Err(LuaError::InvalidArithmetic(rhs)),
            Value::NativeFunction(_) => Err(LuaError::InvalidArithmetic(rhs)),
        },
        Value::String(_) => todo!("Implement string, string operations"),
        Value::Boolean(_) => Err(LuaError::InvalidArithmetic(lhs)),
        Value::Table(_) => Err(LuaError::InvalidArithmetic(lhs)),
        Value::Function(_) => Err(LuaError::InvalidArithmetic(lhs)),
        Value::NativeFunction(_) => Err(LuaError::InvalidArithmetic(lhs)),
    }
}

pub fn execute_logic_operation(lhs: Value,
                               rhs: Value,
                               number_operation: fn(f64, f64) -> bool) -> Value {
    match lhs {
        Value::Nil => Value::Nil,
        Value::Number(lhs_n) => match rhs {
            Value::Nil => Value::Nil,
            Value::Number(rhs_n) => Value::Boolean(number_operation(lhs_n, rhs_n)),
            Value::String(_) => Value::Nil,
            Value::Boolean(_) => Value::Nil,
            Value::Table(_) => Value::Nil,
            Value::Function(_) => Value::Nil,
            Value::NativeFunction(_) => Value::Nil,
        },
        Value::String(_) => Value::Nil,
        Value::Boolean(_) => Value::Nil,
        Value::Table(_) => Value::Nil,
        Value::Function(_) => Value::Nil,
        Value::NativeFunction(_) => Value::Nil,
    }
}
