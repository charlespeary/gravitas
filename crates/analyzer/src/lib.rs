use common::Symbol;
use fun::Function;
use parser::parse::ProgramErrors;
use parser::{
    parse::{
        expr::{atom::AtomicValue, Expr, ExprKind},
        stmt::{Stmt, StmtKind},
        Program, Span,
    },
    utils::error::{ParseError, ParseErrorCause},
};
use std::collections::{HashMap, HashSet};

pub(crate) mod fun;

pub type AnalyzerResult = Result<(), ParseError>;

#[derive(Default)]
pub struct Analyzer {
    variables: HashMap<Symbol, bool>,
    classes: HashSet<Symbol>,
    functions: HashMap<Symbol, Function>,
    in_loop: bool,
    in_class: bool,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn err(&mut self, span: Span, cause: ParseErrorCause) -> AnalyzerResult {
        Err(ParseError { span, cause })
    }

    fn visit_expr(&mut self, expr: &Expr) -> AnalyzerResult {
        use ExprKind::*;
        let span = expr.span.clone();

        let err = move |cause: ParseErrorCause| Err(ParseError { span, cause });

        match &*expr.kind {
            Atom(value) => {
                if let AtomicValue::Identifier(ident) = value {
                    match self.variables.get(ident) {
                        Some(false) => {
                            return err(ParseErrorCause::UsedBeforeInitialization(*ident));
                        }
                        _ => {
                            return err(ParseErrorCause::NotDefined(*ident));
                        }
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
            Call { callee, args } => {
                if let Atom(AtomicValue::Identifier(callee_name)) = *callee.kind {
                    if !self.functions.contains_key(&callee_name) {
                        return self.err(
                            callee.span.clone(),
                            ParseErrorCause::NotDefined(callee_name),
                        );
                    }
                };
            }
            Closure { params, body } => {}
            Return { value } => {}
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
            Array { values } => {}
            Index { target, position } => {}
            Property { target, paths } => {}
            Super | This => {
                if !self.in_class {
                    return err(ParseErrorCause::UsedOutsideClass);
                }
            }
        }
        Ok(())
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> AnalyzerResult {
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
            FunctionDeclaration { body, params, name } => {
                self.declare_function(*name, params.kind.len());
            }
            Expression { expr } => {
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
