use common::Symbol;
use parser::{
    parse::{
        expr::{atom::AtomicValue, Expr, ExprKind},
        stmt::{Stmt, StmtKind},
        AstRef,
    },
    utils::error::{ParseError, ParseErrorCause},
};
use std::collections::{HashMap, HashSet};

pub type AnalyzerResult<E> = Result<(), E>;

#[derive(Default)]
pub struct Analyzer {
    variables: HashMap<Symbol, bool>,
    classes: HashSet<Symbol>,
    in_loop: bool,
    in_class: bool,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn visit_expr(&mut self, expr: &Expr) -> AnalyzerResult<ParseError> {
        use ExprKind::*;
        let span = expr.span.clone();

        let err = move |cause: ParseErrorCause| Err(ParseError { span, cause });

        match &*expr.kind {
            Atom(AtomicValue::Identifier(ident)) => match self.variables.get(ident) {
                Some(false) => {
                    return err(ParseErrorCause::UsedBeforeInitialization(*ident));
                }
                Some(true) => {}
                None => {
                    return err(ParseErrorCause::NotDefined(*ident));
                }
            },
            Binary { lhs, rhs, .. } => {
                self.visit_expr(lhs)?;
                self.visit_expr(rhs)?;
            }
            While { condition, body } => {
                self.visit_expr(condition)?;
                self.in_loop = true;
                self.visit_expr(body)?;
                self.in_loop = false;
            }
            Continue => {
                if !self.in_loop {
                    return err(ParseErrorCause::UsedOutsideLoop);
                }
            }
            Break { return_expr } => {
                if !self.in_loop {
                    return err(ParseErrorCause::UsedOutsideLoop);
                }

                if let Some(expr) = return_expr {
                    self.visit_expr(expr)?;
                }
            }
            Super | This => {
                if !self.in_class {
                    return err(ParseErrorCause::UsedOutsideClass);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> AnalyzerResult<ParseError> {
        use StmtKind::*;

        let span = stmt.span.clone();
        let err = move |cause: ParseErrorCause| Err(ParseError { span, cause });

        match &*stmt.kind {
            VariableDeclaration { name, expr } => {
                self.variables.insert(*name, false);
                self.visit_expr(expr)?;
                self.variables.insert(*name, true);
            }
            ClassDeclaration {
                name,
                super_class,
                methods,
                properties,
            } => {
                self.classes.insert(*name);

                if let Some(supclass) = super_class {
                    if supclass == name {
                        return err(ParseErrorCause::CantInheritFromItself);
                    }

                    if !self.classes.contains(supclass) {
                        return err(ParseErrorCause::SuperclassDoesntExist);
                    }
                }

                self.in_class = true;

                for property in properties {
                    self.visit_stmt(property)?;
                }

                for method in methods {
                    self.visit_stmt(method)?;
                }

                self.in_class = false;
            }
            FunctionDeclaration { body, name, .. } => {
                self.variables.insert(*name, false);
                self.visit_expr(body)?;
                self.variables.insert(*name, true);
            }
            Expression { expr } => {
                self.visit_expr(expr)?;
            }
        }
        Ok(())
    }

    pub fn analyze(&mut self, ast: AstRef) -> AnalyzerResult<Vec<ParseError>> {
        let mut errors: Vec<ParseError> = Vec::new();

        for stmt in ast {
            if let Err(e) = self.visit_stmt(stmt) {
                errors.push(e);
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(())
        }
    }
}

pub fn analyze(ast: AstRef) -> AnalyzerResult<Vec<ParseError>> {
    let mut analyzer = Analyzer::new();
    analyzer.analyze(&ast)?;
    Ok(())
}
