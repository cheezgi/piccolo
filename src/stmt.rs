
use token::Token;

pub trait StmtAccept {
    fn accept<T: StmtVisitor>(&self, visitor: &mut T) -> T::Output;
}

pub trait StmtVisitor {
    type Output;
    fn visit_expr(&mut self, e: &StmtExpr) -> Self::Output;
    fn visit_me_tmp(&mut self, m: &MeTmp) -> Self::Output;
    fn visit_assignment(&mut self, a: &Assignment) -> Self::Output;
}

#[derive(Debug, PartialEq, Clone)]
pub struct MeTmp(pub ::expr::Expr);

impl StmtAccept for MeTmp {
    fn accept<T: StmtVisitor>(&self, v: &mut T) -> T::Output {
        v.visit_me_tmp(&self)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StmtExpr(pub ::expr::Expr);

impl StmtAccept for StmtExpr {
    fn accept<T: StmtVisitor>(&self, v: &mut T) -> T::Output {
        v.visit_expr(&self)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assignment {
    pub name: ::token::Token,
    pub value: ::expr::Expr,
}

impl StmtAccept for Assignment {
    fn accept<T: StmtVisitor>(&self, v: &mut T) -> T::Output {
        v.visit_assignment(&self)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    StmtExpr(StmtExpr),
    MeTmp(MeTmp),
    Assignment(Assignment),
}

impl StmtAccept for Stmt {
    fn accept<T: StmtVisitor>(&self, v: &mut T) -> T::Output {
        match *self {
            Stmt::StmtExpr(ref e) => e.accept(v),
            Stmt::MeTmp(ref e) => e.accept(v),
            Stmt::Assignment(ref e) => e.accept(v),
        }
    }
}

