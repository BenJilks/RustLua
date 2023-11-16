use crate::ast::{Program, Statement, Expression, Term, Operation, Function};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Number(i32),
    Function(Vec<String>, Vec<Statement>),
    NativeFunction(fn(Vec<Value>) -> Value),
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
            Statement::Expression(expression) => { self.execute_expression(local_scope, expression); None },
            Statement::Return(value) => Some(self.execute_expression(local_scope, value)),
            Statement::Local(name, value) => { self.execute_local(local_scope, name, value); None },
            Statement::Function(function) => { self.execute_function(function); None },
        }
    }

    fn execute_local(&mut self, local_scope: &mut Scope, name: &str, value: &Box<Expression>) {
        let evaluated_value = self.execute_expression(local_scope, value);
        local_scope.insert(name.to_owned(), evaluated_value);
    }

    fn execute_function(&mut self, function: &Function) {
        let function_value = Value::Function(function.parameters.clone(), function.body.clone());
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

            Expression::Assignment(name, value) => {
                let evaluated_value = self.execute_expression(local_scope, value);
                self.global_scope.insert(name.to_owned(), evaluated_value.clone());
                evaluated_value
            },
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
                Value::Function(_, _) => Value::Nil,
                Value::NativeFunction(_) => Value::Nil,
            },
            Value::Function(_, _) => Value::Nil,
            Value::NativeFunction(_) => Value::Nil,
        }
    }

    fn execute_term(&mut self, local_scope: &mut Scope, term: &Term) -> Value {
        match term {
            Term::Number(n) => Value::Number(*n),
            Term::Variable(identifier) => self.get(&local_scope, identifier).unwrap_or(&Value::Nil).clone(),
            Term::Call(callee, arguments) => self.execute_call(local_scope, callee, arguments),
        }
    }

    fn execute_call<'a>(&mut self,
                        local_scope: &mut Scope,
                        callee: &String,
                        arguments: &Vec<Box<Expression>>) -> Value {
        let evaluated_callee_or_none = self.get(&local_scope, callee);
        if evaluated_callee_or_none.is_none() {
            return Value::Nil;
        }

        let evaluated_callee = evaluated_callee_or_none.unwrap().clone();
        match evaluated_callee {
            Value::NativeFunction(func) =>
                self.execute_native_call(local_scope, arguments, func),

            Value::Function(parameters, body) =>
                self.execute_function_call(local_scope, arguments, parameters, body),

            _ => Value::Nil,
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
                                 parameters: Vec<String>,
                                 body: Vec<Statement>) -> Value {
        if parameters.len() != arguments.len() {
            todo!("Throw error");
        }

        let mut function_scope = Scope::default();
        for (argument, parameter) in arguments.iter().zip(parameters) {
            function_scope.insert(parameter.to_owned(), self.execute_expression(local_scope, argument));
        }

        for statement in body {
            if let Some(return_value) = self.execute_statement(&mut function_scope, &statement) {
                return return_value;
            }
        }

        Value::Nil
    }
}
