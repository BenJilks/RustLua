
pub type Program = Vec<Statement>;

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Assignment(Box<Expression>, Box<Expression>),
    Return(Box<Expression>),
    Local(String, Box<Expression>),
    Expression(Box<Expression>),
    Function(Function),
    If(Box<Expression>, Vec<Statement>, Vec<(Box<Expression>, Vec<Statement>)>, Option<Vec<Statement>>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,

    Equals,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Term(Term),
    Binary(Box<Expression>, Operation, Box<Expression>),
    Call(Box<Expression>, Vec<Box<Expression>>),
    Dot(Box<Expression>, String),
    Index(Box<Expression>, Box<Expression>),
    Function(Vec<String>, Vec<Statement>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Term {
    Number(f64),
    String(String),
    Boolean(bool),
    Variable(String),
    Table,
}
