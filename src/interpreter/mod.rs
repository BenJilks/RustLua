use crate::ast::{Program, Statement, Expression, Term, Operation, Function};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use value::{Scope, Index, FunctionCapture};
use error::LuaError;

pub use value::Value;
pub type Result<T> = std::result::Result<T, LuaError>;

mod value;
mod error;

pub struct Interpreter {
    global_scope: Scope,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter { global_scope: Scope::default() }
    }

    pub fn execute(&mut self, program: Program) -> Result<()> {
        let mut scope = Scope::default();
        for statement in program {
            self.execute_statement(&mut scope, &statement)?;
        }

        Ok(())
    }

    pub fn define(&mut self, name: &str, func: fn(Vec<Value>) -> Value) {
        self.global_scope.put(name.to_owned(), Value::NativeFunction(func));
    }

    fn execute_statement(&mut self, scope: &mut Scope, statement: &Statement) -> Result<Option<Value>> {
        Ok(match statement {
            Statement::Assignment(lhs, rhs) => { self.execute_assign(scope, lhs, rhs)?; None },
            Statement::Expression(expression) => { self.execute_expression(scope, expression)?; None },
            Statement::Return(value) => Some(self.execute_expression(scope, value)?),
            Statement::Local(name, value) => { self.execute_local(scope, name, value)?; None },
            Statement::Function(function) => { self.execute_function(scope, function); None },
        })
    }

    fn execute_local(&mut self, scope: &mut Scope, name: &str, value: &Box<Expression>) -> Result<()> {
        let evaluated_value = self.execute_expression(scope, value)?;
        scope.put(name.to_owned(), evaluated_value);
        Ok(())
    }

    fn execute_function(&mut self, scope: &mut Scope, function: &Function) {
        let function_value = Value::Function(Rc::from(FunctionCapture {
            parameters: function.parameters.clone(),
            body: function.body.clone(),
            capture: scope.clone(),
        }));

        self.global_scope.put(function.name.clone(), function_value);
    }

    fn execute_expression(&mut self, scope: &mut Scope, expression: &Box<Expression>) -> Result<Value> {
        Ok(match expression.as_ref() {
            Expression::Term(term) => self.execute_term(scope, term),
            Expression::Binary(lhs, operation, rhs) => {
                let lhs = self.execute_expression(scope, lhs)?;
                let rhs = self.execute_expression(scope, rhs)?;
                match operation {
                    Operation::Add => value::execute_arithmetic_operation(lhs, rhs, |a, b| a + b),
                    Operation::Subtract => value::execute_arithmetic_operation(lhs, rhs, |a, b| a - b),
                    Operation::Multiply => value::execute_arithmetic_operation(lhs, rhs, |a, b| a * b),
                    Operation::Divide => value::execute_arithmetic_operation(lhs, rhs, |a, b| a / b),

                    Operation::Equals => value::execute_logic_operation(lhs, rhs, |a, b| a == b),
                }
            },

            Expression::Function(parameters, body) => Value::Function(Rc::from(FunctionCapture {
                parameters: parameters.clone(),
                body: body.clone(),
                capture: scope.clone(),
            })),

            Expression::Call(callee, arguments) => self.execute_call(scope, callee, arguments)?,
            Expression::Dot(value, name) => self.execute_dot_operation(scope, value, name)?,
            Expression::Index(value, index) => self.execute_index_operation(scope, value, index)?,
        })
    }

    fn execute_assign(&mut self, scope: &mut Scope, lhs: &Box<Expression>, rhs: &Box<Expression>) -> Result<()> {
        let evaluated_value = self.execute_expression(scope, rhs)?;

        match lhs.as_ref() {
            Expression::Term(Term::Variable(name)) => {
                if scope.has(name) {
                    scope.put(name.to_owned(), evaluated_value);
                } else {
                    self.global_scope.put(name.to_owned(), evaluated_value);
                }
            },

            Expression::Dot(table, name) => {
                let table = self.execute_expression(scope, table)?;
                match table {
                    Value::Table(table) => {
                        table.borrow_mut().insert(Index::Name(name.to_owned()), evaluated_value);
                    },

                    _ => return Err(LuaError::InvalidIndex(table)),
                }
            },

            Expression::Index(table, index) => {
                let table = self.execute_expression(scope, table)?;
                match table {
                    Value::Table(table) => {
                        let index = self.evaluate_index(scope, index)?;
                        table.borrow_mut().insert(index, evaluated_value);
                    },

                    _ => return Err(LuaError::InvalidIndex(table)),
                }
            },

            _ => todo!("Throw error"),
        }

        Ok(())
    }

    fn execute_dot_operation(&mut self, scope: &mut Scope, value: &Box<Expression>, name: &str) -> Result<Value> {
        let evaluated_value = self.execute_expression(scope, value)?;
        let index = Index::Name(name.to_owned());

        match evaluated_value {
            Value::Table(table) => {
                Ok(table.borrow().get(&index).unwrap_or(&Value::Nil).clone())
            },

            _ => Err(LuaError::InvalidIndex(evaluated_value)),
        }
    }

    fn execute_index_operation(&mut self, scope: &mut Scope, value: &Box<Expression>, index: &Box<Expression>) -> Result<Value> {
        let evaluated_value = self.execute_expression(scope, value)?;
        let index = self.evaluate_index(scope, index)?;

        match evaluated_value {
            Value::Table(table) => {
                Ok(table.borrow().get(&index).unwrap_or(&Value::Nil).clone())
            },

            _ => Err(LuaError::InvalidIndex(evaluated_value)),
        }
    }

    fn evaluate_index(&mut self, scope: &mut Scope, index: &Box<Expression>) -> Result<Index> {
        let evaluated_index = self.execute_expression(scope, index)?;
        match evaluated_index {
            Value::Number(n) => {
                if f64::trunc(n) == n {
                    Ok(Index::Number(n as i32))
                } else {
                    Ok(Index::Name(n.to_string()))
                }
            },

            Value::String(s) => Ok(Index::Name(s)),

            // FIXME: We should be able to use anything as an index.
            _ => todo!("Throw error"),
        }
    }

    fn execute_term(&mut self, scope: &mut Scope, term: &Term) -> Value {
        match term {
            Term::Number(n) => Value::Number(*n),
            Term::String(s) => Value::String(s.to_owned()),
            Term::Boolean(b) => Value::Boolean(*b),
            Term::Variable(identifier) => {
                scope.get(identifier)
                    .unwrap_or(self.global_scope.get(identifier)
                    .unwrap_or(Value::Nil))
            },
            Term::Table => self.execute_construct_table(),
        }
    }

    fn execute_construct_table(&self) -> Value {
        Value::Table(Rc::new(RefCell::new(HashMap::new())))
    }

    fn execute_call<'a>(&mut self,
                        scope: &mut Scope,
                        callee: &Box<Expression>,
                        arguments: &Vec<Box<Expression>>) -> Result<Value> {
        let evaluated_callee = self.execute_expression(scope, callee)?;
        match evaluated_callee {
            Value::NativeFunction(func) =>
                self.execute_native_call(scope, arguments, func),

            Value::Function(function_capture) =>
                self.execute_function_call(scope, arguments, &function_capture),

            _ => Err(LuaError::InvalidCall(evaluated_callee)),
        }
    }

    fn execute_native_call<'a>(&mut self,
                               scope: &mut Scope,
                               arguments: &Vec<Box<Expression>>,
                               func: fn(Vec<Value>) -> Value) -> Result<Value> {
        Ok(func(arguments
            .iter()
            .map(|argument| self.execute_expression(scope, argument))
            .collect::<Result<Vec<_>>>()?))
    }

    fn execute_function_call<'a>(&mut self,
                                 scope: &mut Scope,
                                 arguments: &Vec<Box<Expression>>,
                                 function_capture: &FunctionCapture) -> Result<Value> {
        let parameters = &function_capture.parameters;
        let body = &function_capture.body;
        if parameters.len() != arguments.len() {
            // FIXME: This should be allowed
            todo!("Throw error");
        }

        let mut function_scope = function_capture.capture.clone();
        for (argument, parameter) in arguments.iter().zip(parameters) {
            function_scope.put(parameter.to_owned(), self.execute_expression(scope, argument)?);
        }

        for statement in body {
            if let Some(return_value) = self.execute_statement(&mut function_scope, &statement)? {
                return Ok(return_value);
            }
        }

        Ok(Value::Nil)
    }
}
