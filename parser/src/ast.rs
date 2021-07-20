#[derive(Debug)]
pub struct Ast {
    pub sequence: Vec<Compound>,
}

#[derive(Debug)]
pub enum Compound {
    Statement,
    Expression(Expression),
}

#[derive(Debug)]
pub enum Identifier {
    Variable(Variable), // Should be expaned to variable value. Must be done before glob.
    Glob(String),       // Should be glob expanded.
    Text(String),
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
}

#[derive(Debug)]
pub enum Expression {
    Call(Call),
    Redirect,
    Pipe,
    Math,
}

#[derive(Debug)]
pub struct Call {
    pub args: Vec<Argument>,
}

#[derive(Debug)]
pub struct Pipe {
    pub source: Expression,
    pub destination: Call,
}

#[derive(Debug)]
pub struct Argument {
    pub parts: Vec<Identifier>,
}

#[derive(Debug)]
pub struct Redirect {
    pub call: Expression,
    pub file: Identifier,
}

#[derive(Debug)]
pub struct Assignment {
    pub variable: Variable,
    pub expression: Expression,
}

#[derive(Debug)]
pub enum Statement {
    _Assignment,
    _If,
    _Fn,
    _Loop,
}
