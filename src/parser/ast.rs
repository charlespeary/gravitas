use anyhow::Result;
use derive_more::Display;
use enum_as_inner::EnumAsInner;

use crate::parser::Token;
use crate::utils::Either;

#[derive(Debug, Display, PartialEq)]
pub enum Atom {
    Text(String),
    Number(f64),
    Bool(bool),
    Null,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BranchType {
    If,
    ElseIf,
    Else,
}

#[derive(Debug, PartialEq)]
pub struct IfBranch {
    pub condition: Expr,
    pub body: Block,
    pub branch_type: BranchType,
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub body: Vec<Stmt>,
    pub final_expr: Option<Box<Expr>>,
}

impl Into<Expr> for Block {
    fn into(self) -> Expr {
        Expr::Block { body: self }
    }
}

pub enum FunctionType {
    Closure,
    Standard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arg(pub String);

#[derive(Debug, PartialEq)]
pub struct Ast(pub Vec<Stmt>);

pub type ExprOrStmt = Either<Stmt, Expr>;

#[derive(Debug, PartialEq, EnumAsInner)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Var {
        identifier: String,
        is_ref: bool,
    },
    Unary {
        expr: Box<Expr>,
        operator: Token,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Block {
        body: Block,
    },
    If {
        branches: Vec<IfBranch>,
    },
    While {
        condition: Box<Expr>,
        body: Block,
    },
    Break {
        expr: Option<Box<Expr>>,
    },
    Continue,
    Atom(Atom),
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    // Expressions
    Expr {
        expr: Expr,
    },
    // Declarations
    Var {
        expr: Expr,
        identifier: String,
    },
    // Side effects
    Print {
        expr: Expr,
    },
    Function {
        args: Vec<Arg>,
        body: Block,
        name: String,
    },
}

pub trait Visitor<T> {
    type Item;
    fn visit(&mut self, t: &T) -> Result<Self::Item>;
}

pub trait Visitable: Sized {
    fn accept<T>(&self, t: &mut T) -> Result<T::Item>
    where
        T: Visitor<Self>,
    {
        t.visit(self)
    }
}

impl Visitable for Ast {}

impl Visitable for Expr {}

impl Visitable for Stmt {}

impl Visitable for Block {}
