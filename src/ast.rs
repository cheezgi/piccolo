
//use ::expr::*;

#[derive(Debug, PartialEq)]
pub struct Paren {
    pub inner: Expression,
}

#[derive(Debug, PartialEq)]
pub struct Access {
    pub from: Expression,
    pub of: Expression,
}

#[derive(Debug, PartialEq)]
pub struct Variable {
    pub name: String,
    pub value: Expression, // TODO
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    String(String),
    Integer(i64),
    Float(f64),
    True,
    False,
    Paren(Paren),
}

#[derive(Debug, PartialEq)]
pub struct FunctionCall {
    pub callee: Expression,
    pub args: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, PartialEq)]
pub struct UnaryRest {
    pub op: UnaryOp,
    pub rhs: Box<Unary>,
}

#[derive(Debug, PartialEq)]
pub enum Unary {
    Primary(Literal),
    UnaryOp(UnaryRest),
}

#[derive(Debug, PartialEq)]
pub enum MultiplicationOp {
    Multiply,
    Divide,
    Mod,
    And,
    Or,
    Xor,
}

#[derive(Debug, PartialEq)]
pub struct MultiplicationRest {
    pub op: MultiplicationOp,
    pub rhs: Vec<Unary>,
}

#[derive(Debug, PartialEq)]
pub struct Multiplication {
    pub unary: Unary,
    pub rest: MultiplicationRest,
}

#[derive(Debug, PartialEq)]
pub enum AdditionOp {
    Plus,
    Minus,
}

#[derive(Debug, PartialEq)]
pub struct AdditionRest {
    pub op: AdditionOp,
    pub rhs: Multiplication,
}

#[derive(Debug, PartialEq)]
pub struct Addition {
    pub addition: Multiplication,
    pub rest: Vec<AdditionRest>,
}

#[derive(Debug, PartialEq)]
pub enum ComparisonOp {
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
}

#[derive(Debug, PartialEq)]
pub struct ComparisonRest {
    pub op: ComparisonOp,
    pub rhs: Addition
}

#[derive(Debug, PartialEq)]
pub struct Comparison {
    pub addition: Addition,
    pub rest: Vec<ComparisonRest>
}

#[derive(Debug, PartialEq)]
pub enum EqualityOp {
    NotEqual,
    Equal,
}

#[derive(Debug, PartialEq)]
pub struct EqualityRest {
    pub op: EqualityOp,
    pub rhs: Comparison,
}

#[derive(Debug, PartialEq)]
pub struct Equality {
    pub comparison: Comparison,
    pub rest: Vec<ComparisonRest>,
}

#[derive(Debug, PartialEq)]
pub struct OrRest {
    pub rhs: Equality,
}

#[derive(Debug, PartialEq)]
pub struct Or {
    pub equality: Equality,
    pub rest: Vec<EqualityRest>,
}

#[derive(Debug, PartialEq)]
pub struct AndRest {
    pub rhs: Or,
}

#[derive(Debug, PartialEq)]
pub struct And {
    pub or: Or,
    pub rest: Vec<OrRest>,
}

#[derive(Debug, PartialEq)]
pub struct Math {
    pub and: And,
    pub rest: Vec<AndRest>,
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    //Paren(Box<Paren>),
    Access(Box<Access>),
    Variable(Box<Variable>),
    Literal(Box<Literal>),
    FunctionCall(Box<FunctionCall>),
    Math(Box<Math>),
    Nil,
}

#[derive(Debug, PartialEq)]
pub struct Assignment {
    pub name: String,
    pub expr: Expression,
}

#[derive(Debug, PartialEq)]
pub struct Return {
    pub value: Expression,
}

#[derive(Debug, PartialEq)]
pub struct Error {
    pub msg: Option<String>,
    pub value: Expression,
}

#[derive(Debug, PartialEq)]
pub enum FlowKind {
    If,
    While,
    For,
}

#[derive(Debug, PartialEq)]
pub struct Flow {
    pub flow_kind: FlowKind,
    pub inner: Vec<Statement>,
    pub else_: Option<Expression>
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Expression(Box<Expression>),
    Assignment(Box<Assignment>),
    Return(Box<Return>),
    Error(Box<Error>),
    Flow(Box<Flow>),
    Block(Vec<Statement>),
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub args: Vec<String>,
    pub inner: Vec<Statement>,
}

#[derive(Debug, PartialEq)]
pub struct Data {
    pub name: String,
    pub fields: Option<Vec<Assignment>>,
    pub methods: Option<Vec<Function>>,
}

#[derive(Debug, PartialEq)]
pub enum Piccolo {
    Statement(Box<Statement>),
    Function(Box<Function>),
    Data(Box<Data>),
}

#[derive(Debug, PartialEq)]
pub struct Ast {
    pub inner: Vec<Piccolo>,
}
