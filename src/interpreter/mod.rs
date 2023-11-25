use crate::ast::{Statement, Expression, Term, Operation, Function, TableConstructionIndex};
use crate::lua_parser;
use std::rc::Rc;
use std::cell::RefCell;
use value::{Scope, Index, Table, FunctionCapture};

pub use value::Value;
pub use error::LuaError;
pub type Result<T> = std::result::Result<T, LuaError>;

mod value;
mod error;

pub struct Interpreter {
    global_scope: Scope,
    parser: lua_parser::ProgramParser,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            global_scope: Scope::default(),
            parser: lua_parser::ProgramParser::new(),
        }
    }

    pub fn execute(&mut self, source: &str) -> Result<Value> {
        let program = self.parser.parse(source).unwrap();

        let mut scope = Scope::default();
        Ok(self.execute_body(&mut scope, &program)?.unwrap_or(Value::Nil))
    }

    pub fn define(&mut self, name: &str, func: fn(Vec<Value>) -> Value) {
        self.global_scope.put(name.to_owned(), Value::NativeFunction(func));
    }

    fn execute_body(&mut self, scope: &mut Scope, body: &Vec<Statement>) -> Result<Option<Value>> {
        for statement in body {
            if let Some(value) = self.execute_statement(scope, &statement)? {
                return Ok(Some(value))
            }
        }

        Ok(None)
    }

    fn execute_statement(&mut self, scope: &mut Scope, statement: &Statement) -> Result<Option<Value>> {
        Ok(match statement {
            Statement::Assignment(lhs, rhs) => { self.execute_assign(scope, lhs, rhs)?; None },
            Statement::Expression(expression) => { self.execute_expression(scope, expression)?; None },
            Statement::Return(value) => Some(self.execute_expression(scope, value)?),
            Statement::Local(name, value) => { self.execute_local(scope, name, value)?; None },
            Statement::Function(function) => { self.execute_function(scope, function); None },

            Statement::If(condition, then, elseif, else_) =>
                self.execute_if(scope, condition, then, elseif, else_)?,

            Statement::NumericFor(name, start, end, step, body) =>
                self.execute_numeric_for(scope, name, start, end, step, body)?,
        })
    }

    fn execute_numeric_for(&mut self,
                           scope: &mut Scope,
                           name: &String,
                           start: &Box<Expression>,
                           end: &Box<Expression>,
                           step: &Option<Box<Expression>>,
                           body: &Vec<Statement>) -> Result<Option<Value>> {
        let evaluated_start = self.execute_expression(scope, start)?;
        let mut value = match evaluated_start {
            Value::Number(initial_value) => initial_value,
            value => return Err(LuaError::BadForInitialValue(value)),
        };

        let evaluated_end = self.execute_expression(scope, end)?;
        let limit = match evaluated_end {
            Value::Number(limit) => limit,
            value => return Err(LuaError::BadForLimit(value)),
        };

        let evaluated_step = step.as_ref()
            .map(|step| self.execute_expression(scope, step));
        let step = match evaluated_step {
            Some(Ok(Value::Number(step))) => step,
            Some(Ok(value)) => return Err(LuaError::BadForStep(value)),
            Some(Err(err)) => return Err(err),
            None => 1.0,
        };

        while value <= limit {
            scope.put(name.to_owned(), Value::Number(value));
            if let Some(value) = self.execute_body(scope, body)? {
                return Ok(Some(value));
            }

            value += step;
        }

        Ok(None)
    }

    fn execute_if(&mut self,
                  scope: &mut Scope,
                  condition: &Box<Expression>,
                  then: &Vec<Statement>,
                  elseif: &Vec<(Box<Expression>, Vec<Statement>)>,
                  else_: &Option<Vec<Statement>>) -> Result<Option<Value>> {
        let evaluated_condition = self.execute_expression(scope, condition)?;
        if evaluated_condition.is_truthy() {
            return self.execute_body(scope, then);
        }

        for (condition, then) in elseif {
            let evaluated_condition = self.execute_expression(scope, condition)?;
            if evaluated_condition.is_truthy() {
                return self.execute_body(scope, then);
            }
        }

        match else_ {
            Some(body) => self.execute_body(scope, body),
            None => Ok(None),
        }
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
            Expression::Term(term) => self.execute_term(scope, term)?,
            Expression::Binary(lhs, operation, rhs) => {
                let lhs = self.execute_expression(scope, lhs)?;
                let rhs = self.execute_expression(scope, rhs)?;
                match operation {
                    Operation::Add => value::execute_arithmetic_operation(lhs, rhs, |a, b| a + b)?,
                    Operation::Subtract => value::execute_arithmetic_operation(lhs, rhs, |a, b| a - b)?,
                    Operation::Multiply => value::execute_arithmetic_operation(lhs, rhs, |a, b| a * b)?,
                    Operation::Divide => value::execute_arithmetic_operation(lhs, rhs, |a, b| a / b)?,

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

    fn execute_term(&mut self, scope: &mut Scope, term: &Term) -> Result<Value> {
        Ok(match term {
            Term::Number(n) => Value::Number(*n),
            Term::String(s) => Value::String(s.to_owned()),
            Term::Boolean(b) => Value::Boolean(*b),
            Term::Variable(identifier) => {
                scope.get(identifier)
                    .unwrap_or(self.global_scope.get(identifier)
                    .unwrap_or(Value::Nil))
            },
            Term::Table(items) => self.execute_construct_table(scope, items)?,
        })
    }

    fn execute_construct_table(&mut self,
                               scope: &mut Scope,
                               items: &Vec<(Option<TableConstructionIndex>, Box<Expression>)>) -> Result<Value> {
        let mut table = Table::default();
        let mut current_numeric_index = 1i32;

        for (index, value) in items {
            let value = self.execute_expression(scope, value)?;
            let index = match index {
                Some(TableConstructionIndex::Name(name)) => Index::Name(name.to_owned()),
                Some(TableConstructionIndex::Value(index)) => self.evaluate_index(scope, index)?,
                None => {
                    let index = Index::Number(current_numeric_index);
                    current_numeric_index += 1;
                    index
                },
            };

            table.insert(index, value);
        }

        Ok(Value::Table(Rc::new(RefCell::new(table))))
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

        Ok(self.execute_body(&mut function_scope, body)?.unwrap_or(Value::Nil))
    }
}
