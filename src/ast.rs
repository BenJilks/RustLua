
pub type Program = Vec<Statement>;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Return(Box<Expression>),
    Local(String, Box<Expression>),
    Expression(Box<Expression>),
    Function(Function),
}

#[derive(Debug, Clone)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Term(Term),
    Binary(Box<Expression>, Operation, Box<Expression>),
    Assignment(String, Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum Term {
    Number(i32),
    Variable(String),
    Call(String, Vec<Box<Expression>>),
}