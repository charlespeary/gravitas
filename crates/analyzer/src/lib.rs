use common::Symbol;
use parser::parse::ProgramErrors;
use parser::{
    parse::{
        expr::{atom::AtomicValue, Expr, ExprKind},
        stmt::{Stmt, StmtKind},
        Program, Span,
    },
    utils::error::{ParseError, ParseErrorCause},
};
use std::collections::HashMap;

pub type AnalyzerResult = Result<(), ParseError>;

#[derive(Default)]
pub struct Analyzer {
    variables: HashMap<Symbol, bool>,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn error(&mut self, span: Span, cause: ParseErrorCause) -> AnalyzerResult {
        Err(ParseError { span, cause })
    }

    fn visit_expr(&mut self, expr: &Expr) -> AnalyzerResult {
        use ExprKind::*;
        match &*expr.kind {
            Atom(value) => {
                if let AtomicValue::Identifier(ident) = value {
                    if !self.variables.contains_key(ident) {
                        return self
                            .error(expr.span.clone(), ParseErrorCause::UsedBeforeInitialization);
                    }
                }
            }
            Unary { rhs, op } => {}
            Binary { lhs, op, rhs } => {
                self.visit_expr(lhs)?;
                self.visit_expr(rhs)?;
            }
            Assignment { target, value } => {}
            Block { stmts, return_expr } => {}
            If {
                condition,
                body,
                else_expr,
            } => {}
            Call { callee, args } => {}
            Closure { params, body } => {}
            Return { value } => {}
            While { condition, body } => {}
            Continue => {}
            Break { return_expr } => {}
            Array { values } => {}
            Index { target, position } => {}
            Property { target, paths } => {}
        }
        Ok(())
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> AnalyzerResult {
        match &*stmt.kind {
            StmtKind::VariableDeclaration { name, expr } => {
                self.variables.insert(*name, false);
                self.visit_expr(expr)?;
                self.variables.insert(*name, true);
            }
            StmtKind::ClassDeclaration {
                name,
                super_class,
                methods,
                properties,
            } => {}
            StmtKind::FunctionDeclaration { body, params, name } => {}
            StmtKind::Expression { expr } => {
                self.visit_expr(expr)?;
            }
        }
        Ok(())
    }

    pub fn analyze(&mut self, (ast, symbols): &Program) -> Result<(), Vec<ParseError>> {
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

pub fn analyze(program: Program) -> Result<Program, ProgramErrors> {
    let mut analyzer = Analyzer::new();
    match analyzer.analyze(&program) {
        Ok(_) => Ok(program),
        Err(errors) => Err((errors, program.1)),
    }
}
