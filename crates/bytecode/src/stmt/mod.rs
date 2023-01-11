use crate::{
    callables::Function, chunk::Constant, state::ScopeType, BytecodeFrom, BytecodeGenerationResult,
    BytecodeGenerator, Opcode,
};
use parser::parse::{
    expr::ExprKind,
    stmt::{Stmt, StmtKind},
    FunctionBody, Params,
};

mod var;

impl BytecodeGenerator {
    pub(crate) fn compile_function(
        &mut self,
        name: String,
        params: Params,
        body: FunctionBody,
    ) -> Result<Function, ()> {
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
        self.leave_scope();

        return Ok(new_fn);
    }
}

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
                let new_fn = self.compile_function(name.clone(), params, body)?;
                self.write_constant(Constant::Function(new_fn));
                self.state.declare_var(name);
            }
            StmtKind::ClassDeclaration {
                name,
                super_class,
                methods,
            } => {
                self.enter_scope(ScopeType::Class);

                // To allow methods calling class constructor
                self.state.declare_var(name.clone());

                let mut compiled_methods = vec![];

                for method in methods {
                    if let StmtKind::FunctionDeclaration { name, params, body } = *method.kind {
                        let compiled_method = self.compile_function(name, params, body)?;
                        compiled_methods.push(compiled_method);
                    } else {
                        panic!("Analyzer didn't prevent items that are not function declarations from getting there!");
                    }
                }

                self.leave_scope();
            }
        }
        Ok(())
    }
}
