use crate::ast::{Program, Statement, Expression, Term, Operation, Function};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Index {
    Name(String),
    Number(i32),
}

#[derive(Debug, Clone)]
pub struct FunctionCapture {
    parameters: Vec<String>,
    body: Vec<Statement>,
    capture: Scope,
}

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Number(i32),
    String(String),
    Function(Rc<RefCell<FunctionCapture>>),
    NativeFunction(fn(Vec<Value>) -> Value),
    Table(Rc<RefCell<HashMap<Index, Value>>>),
}

type Scope = HashMap<String, Value>;

pub struct Interpreter {
    global_scope: Scope,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter { global_scope: Scope::default() }
    }

    pub fn execute(&mut self, program: Program) {
        let mut local_scope = Scope::default();
        for statement in program {
            self.execute_statement(&mut local_scope, &statement);
        }
    }

    pub fn define(&mut self, name: &str, func: fn(Vec<Value>) -> Value) {
        self.global_scope.insert(name.to_owned(), Value::NativeFunction(func));
    }

    fn get<'a>(&'a mut self, local_scope: &'a Scope, name: &str) -> Option<&'a Value> {
        match local_scope.get(name) {
            Some(value) => Some(value),
            None => self.global_scope.get(name),
        }
    }

    fn execute_statement(&mut self, local_scope: &mut Scope, statement: &Statement) -> Option<Value> {
        match statement {
            Statement::Assignment(lhs, rhs) => { self.execute_assign(local_scope, lhs, rhs); None },
            Statement::Expression(expression) => { self.execute_expression(local_scope, expression); None },
            Statement::Return(value) => Some(self.execute_expression(local_scope, value)),
            Statement::Local(name, value) => { self.execute_local(local_scope, name, value); None },
            Statement::Function(function) => { self.execute_function(&local_scope, function); None },
        }
    }

    fn execute_local(&mut self, local_scope: &mut Scope, name: &str, value: &Box<Expression>) {
        let evaluated_value = self.execute_expression(local_scope, value);
        local_scope.insert(name.to_owned(), evaluated_value);
    }

    fn execute_function(&mut self, local_scope: &Scope, function: &Function) {
        let function_value = Value::Function(Rc::new(RefCell::new(FunctionCapture {
            parameters: function.parameters.clone(),
            body: function.body.clone(),
            capture: local_scope.clone(),
        })));

        self.global_scope.insert(function.name.clone(), function_value);
    }

    fn execute_expression(&mut self, local_scope: &mut Scope, expression: &Box<Expression>) -> Value {
        match expression.as_ref() {
            Expression::Term(term) => self.execute_term(local_scope, term),
            Expression::Binary(lhs, operation, rhs) => {
                let lhs = self.execute_expression(local_scope, lhs);
                let rhs = self.execute_expression(local_scope, rhs);
                match operation {
                    Operation::Add => self.execute_arithmatic_operation(lhs, rhs, |a, b| a + b),
                    Operation::Subtract => self.execute_arithmatic_operation(lhs, rhs, |a, b| a - b),
                    Operation::Multiply => self.execute_arithmatic_operation(lhs, rhs, |a, b| a * b),
                    Operation::Divide => self.execute_arithmatic_operation(lhs, rhs, |a, b| a / b),
                }
            },

            Expression::Function(parameters, body) => Value::Function(Rc::new(RefCell::new(FunctionCapture {
                parameters: parameters.clone(),
                body: body.clone(),
                capture: local_scope.clone(),
            }))),

            Expression::Call(callee, arguments) => self.execute_call(local_scope, callee, arguments),
            Expression::Dot(value, name) => self.execute_dot_operation(local_scope, value, name),
            Expression::Index(value, index) => self.execute_index_operation(local_scope, value, index),
        }
    }

    fn execute_assign(&mut self, local_scope: &mut Scope, lhs: &Box<Expression>, rhs: &Box<Expression>) {
        let evaluated_value = self.execute_expression(local_scope, rhs);

        match lhs.as_ref() {
            Expression::Term(Term::Variable(name)) => {
                if local_scope.contains_key(name) {
                    local_scope.insert(name.to_owned(), evaluated_value);
                } else {
                    self.global_scope.insert(name.to_owned(), evaluated_value);
                }
            },

            Expression::Dot(table, name) => {
                let table = self.execute_expression(local_scope, table);
                match table {
                    Value::Table(table) => {
                        table.borrow_mut().insert(Index::Name(name.to_owned()), evaluated_value);
                    },

                    _ => todo!("Throw error"),
                }
            },

            Expression::Index(table, index) => {
                let table = self.execute_expression(local_scope, table);
                match table {
                    Value::Table(table) => {
                        let index = self.evaluate_index(local_scope, index);
                        table.borrow_mut().insert(index, evaluated_value);
                    },

                    _ => todo!("Throw error"),
                }
            },

            _ => todo!("Throw error"),
        }
    }

    fn execute_dot_operation(&mut self, local_scope: &mut Scope, value: &Box<Expression>, name: &str) -> Value {
        let evaluated_value = self.execute_expression(local_scope, value);
        let index = Index::Name(name.to_owned());

        match evaluated_value {
            Value::Table(table) => {
                table.borrow().get(&index).unwrap_or(&Value::Nil).clone()
            },

            _ => todo!("Throw error"),
        }
    }

    fn execute_index_operation(&mut self, local_scope: &mut Scope, value: &Box<Expression>, index: &Box<Expression>) -> Value {
        let evaluated_value = self.execute_expression(local_scope, value);
        let index = self.evaluate_index(local_scope, index);

        match evaluated_value {
            Value::Table(table) => {
                table.borrow().get(&index).unwrap_or(&Value::Nil).clone()
            },

            _ => todo!("Throw error"),
        }
    }

    fn evaluate_index(&mut self, local_scope: &mut Scope, index: &Box<Expression>) -> Index {
        let evaluated_index = self.execute_expression(local_scope, index);
        match evaluated_index {
            Value::Number(n) => Index::Number(n),
            Value::String(s) => Index::Name(s),
            _ => todo!("Throw error"),
        }
    }

    fn execute_arithmatic_operation(&mut self,
                                    lhs: Value,
                                    rhs: Value,
                                    number_operation: fn(i32, i32) -> i32) -> Value {
        match lhs {
            Value::Nil => Value::Nil,
            Value::Number(lhs_n) => match rhs {
                Value::Nil => Value::Nil,
                Value::Number(rhs_n) => Value::Number(number_operation(lhs_n, rhs_n)),
                Value::String(_) => Value::Nil,
                Value::Table(_) => Value::Nil,
                Value::Function(_) => Value::Nil,
                Value::NativeFunction(_) => Value::Nil,
            },
            Value::String(_) => Value::Nil,
            Value::Table(_) => Value::Nil,
            Value::Function(_) => Value::Nil,
            Value::NativeFunction(_) => Value::Nil,
        }
    }

    fn execute_term(&mut self, local_scope: &mut Scope, term: &Term) -> Value {
        match term {
            Term::Number(n) => Value::Number(*n),
            Term::String(s) => Value::String(s.to_owned()),
            Term::Variable(identifier) => self.get(&local_scope, identifier).unwrap_or(&Value::Nil).clone(),
            Term::Table => self.execute_construct_table(),
        }
    }

    fn execute_construct_table(&mut self) -> Value {
        Value::Table(Rc::new(RefCell::new(HashMap::new())))
    }

    fn execute_call<'a>(&mut self,
                        local_scope: &mut Scope,
                        callee: &Box<Expression>,
                        arguments: &Vec<Box<Expression>>) -> Value {
        let evaluated_callee = self.execute_expression(local_scope, callee);
        match evaluated_callee {
            Value::NativeFunction(func) =>
                self.execute_native_call(local_scope, arguments, func),

            Value::Function(function_capture) =>
                self.execute_function_call(local_scope, arguments, function_capture),

            _ => todo!("Throw error"),
        }
    }

    fn execute_native_call<'a>(&mut self,
                               local_scope: &mut Scope,
                               arguments: &Vec<Box<Expression>>,
                               func: fn(Vec<Value>) -> Value) -> Value {
        func(arguments
            .iter()
            .map(|argument| self.execute_expression(local_scope, argument))
            .collect())
    }

    fn execute_function_call<'a>(&mut self,
                                 local_scope: &mut Scope,
                                 arguments: &Vec<Box<Expression>>,
                                 function_capture: Rc<RefCell<FunctionCapture>>) -> Value {
        let mut function_capture = function_capture.borrow_mut();
        let parameters = function_capture.parameters.clone();
        let body = function_capture.body.clone();

        if parameters.len() != arguments.len() {
            todo!("Throw error");
        }

        let function_scope = &mut function_capture.capture;
        for (argument, parameter) in arguments.iter().zip(parameters) {
            function_scope.insert(parameter.to_owned(), self.execute_expression(local_scope, argument));
        }

        for statement in body {
            if let Some(return_value) = self.execute_statement(function_scope, &statement) {
                return return_value;
            }
        }

        Value::Nil
    }
}
