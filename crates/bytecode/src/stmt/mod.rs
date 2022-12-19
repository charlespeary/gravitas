use crate::{chunk::Constant, BytecodeFrom, BytecodeGenerationResult, BytecodeGenerator, Opcode};
use parser::parse::{
    expr::ExprKind,
    stmt::{Stmt, StmtKind},
};

mod var;

impl BytecodeFrom<Stmt> for BytecodeGenerator {
    fn generate(&mut self, stmt: Stmt) -> BytecodeGenerationResult {
        match *stmt.kind {
            StmtKind::Expression { expr } => {
                self.generate(expr)?;
            }
            StmtKind::VariableDeclaration { name, expr } => {
                self.generate(expr)?;
                self.state.declare_var(name);
            }
            StmtKind::FunctionDeclaration { name, params, body } => {
                self.new_function(name.clone(), params.kind.len());

                for param in params.kind {
                    self.state.declare_var(param.kind);
                }

                // To allow recursion
                self.state.declare_var(name.clone());

                match *body.kind {
                    ExprKind::Block { stmts, return_expr } => {
                        self.generate(stmts)?;
                        match return_expr {
                            Some(return_expr) => {
                                self.generate(return_expr)?;
                            }
                            None => {
                                self.write_opcode(Opcode::Null);
                            }
                        };
                        self.write_opcode(Opcode::Return);
                    }
                    _ => {
                        self.generate(body)?;
                        self.write_opcode(Opcode::Return);
                    }
                };

                let new_fn = self
                    .functions
                    .pop()
                    .expect("We just defined and evaluated function. It shouldn't happen.");

                self.write_constant(Constant::Function(new_fn));
                self.state.declare_var(name);
            }
            StmtKind::ClassDeclaration {
                name,
                super_class,
                methods,
            } => {}
        }
        Ok(())
    }
}
