
pub struct Parser {

}

pub enum Compound {
    Variable(Variable), // should be expaned to variable value
    Glob(String), // should be glob expanded 
    String(String),
}

pub struct Variable {
    name: String,
}

pub enum Expression {
    Call(Call),
}

pub struct Call {
    cmd: Vec<Compound>, // () has to be a combination of all identifiers and variables and stuff. Maybe it can be list of tokens? Should evaluate to something callable.
    arguments: Argument,
}

pub struct Pipe {
    source: Expression,
    destination: Call,
}

pub struct Argument {
    parts: Vec<Compound>,
}

pub struct Redirect {
    source: Expression,
    destination: Vec<Compound>,
}