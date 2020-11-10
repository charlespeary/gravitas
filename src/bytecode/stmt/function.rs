use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, Callable, Chunk, GenerationResult, Opcode, Value},
    parser::stmt::function::FunctionStmt,
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: String,
}

impl Into<Value> for Function {
    fn into(self) -> Value {
        Value::Callable(Callable::Function(self))
    }
}

impl BytecodeFrom<FunctionStmt> for BytecodeGenerator {
    fn generate(&mut self, fnc: &FunctionStmt) -> GenerationResult {
        let FunctionStmt { name, params, body } = fnc;
        let mut emitter = self.child();

        // Declare parameters, so they are visible in the body scope
        for param in params.clone() {
            emitter.declare(param.val);
        }

        // Declare function, so we can allow recursive calls.
        // It happens after the parameters, because arguments are evaluated first, then comes the caller value onto the stack.
        emitter.declare(name.clone());

        // We don't want to evaluate block expression, only its items
        for item in &body.body {
            emitter.generate(item)?;
        }

        // Add explicit return with null if user didn't
        if !emitter.state.function_returned {
            emitter.emit_code(Opcode::Null);
            emitter.emit_code(Opcode::Return);
        }

        let function_chunk = emitter.chunk;
        let function = Value::Callable(Callable::Function(Function {
            arity: params.len(),
            chunk: function_chunk,
            name: name.clone(),
        }));

        self.add_constant(function);
        self.declare(name.clone());

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::bytecode::{
        Address, BytecodeFrom, BytecodeGenerator, Chunk,
        Opcode, stmt::function::Function, test::generate_bytecode, Value,
    };
    use crate::parser::{
        expr::{Atom, Binary, Block, Expr, Identifier, Return},
        operator::Operator,
        stmt::{
            expr::ExprStmt,
            function::{FunctionStmt, Param},
            Stmt,
        },
    };

    fn into_function(value: Value) -> Function {
        value
            .into_callable()
            .expect("This is not a callable value")
            .into_function()
            .expect("This is not a function")
    }

    fn generate_function<I>(ast: I) -> Function
        where
            BytecodeGenerator: BytecodeFrom<I>,
    {
        let (chunk, code) = generate_bytecode(ast);
        into_function(chunk.read_constant(0).clone())
    }

    /// Emit correct code for a correct function
    #[test]
    fn correct_function() {
        let ast = Stmt::Function(FunctionStmt {
            name: String::from("foo"),
            params: vec![
                Param {
                    val: String::from("a"),
                },
                Param {
                    val: String::from("b"),
                },
            ],
            body: Block {
                body: vec![Stmt::Expr(ExprStmt {
                    expr: Expr::Return(Return {
                        expr: Some(Box::new(Expr::Atom(Atom::Number(10.0)))),
                    }),
                })],
                final_expr: None,
            },
        });

        let function = generate_function(ast);

        // Generated function has arity corresponding to number of parameters
        assert_eq!(function.arity, 2);
        // Function's name is equal to the one that is parsed
        assert_eq!(function.name, String::from("foo"));
        // Function's chunk is correctly generated from the function's body
        assert_eq!(
            function.chunk,
            Chunk {
                code: vec![Opcode::Constant(0), Opcode::Return, Opcode::PopN(1)],
                constants: vec![Value::Number(10.0)],
            }
        )
    }

    /// Allow calls inside function body to refer to itself
    #[test]
    fn allow_recursion() {
        let ast = Stmt::Function(FunctionStmt {
            name: String::from("foo"),
            params: vec![],
            body: Block {
                body: vec![Stmt::Expr(ExprStmt {
                    expr: Expr::Return(Return {
                        expr: Some(Box::new(Expr::Identifier(Identifier {
                            value: String::from("foo"),
                            is_ref: false,
                        }))),
                    }),
                })],
                final_expr: None,
            },
        });

        let function = generate_function(ast);

        assert_eq!(
            function.chunk,
            Chunk {
                code: vec![
                    Opcode::Constant(0),
                    Opcode::Get,
                    Opcode::Return,
                    Opcode::PopN(1)
                ],
                // Function is always available on the first available address after arguments to enable recursive calls
                // This example doesn't have any arguments, so the function is available at the index 0 of the function's stack
                constants: vec![Value::Address(Address::Local(0))],
            }
        )
    }

    /// Function is able to access passed arguments
    #[test]
    fn use_arguments() {
        let ast = Stmt::Function(FunctionStmt {
            name: String::from("foo"),
            params: vec![
                Param {
                    val: String::from("a"),
                },
                Param {
                    val: String::from("b"),
                },
            ],
            body: Block {
                body: vec![Stmt::Expr(ExprStmt {
                    expr: Expr::Return(Return {
                        expr: Some(Box::new(Expr::Binary(Binary {
                            lhs: Box::new(Expr::Identifier(Identifier {
                                value: String::from("a"),
                                is_ref: false,
                            })),
                            operator: Operator::Plus,
                            rhs: Box::new(Expr::Identifier(Identifier {
                                value: String::from("b"),
                                is_ref: false,
                            })),
                        }))),
                    }),
                })],
                final_expr: None,
            },
        });
        let function = generate_function(ast);
        assert_eq!(
            function.chunk,
            Chunk {
                constants: vec![
                    Value::Address(Address::Local(0)),
                    Value::Address(Address::Local(1))
                ],
                code: vec![
                    Opcode::Constant(0),
                    Opcode::Get,
                    Opcode::Constant(1),
                    Opcode::Get,
                    Opcode::Add,
                    Opcode::Return,
                    Opcode::PopN(1)
                ],
            }
        )
    }

    /// Emit additional opcode to return null from a function without explicit return
    #[test]
    fn without_return() {
        // This function's body is empty, but it still should emit the return null opcodes
        let ast = Stmt::Function(FunctionStmt {
            name: String::from("foo"),
            params: vec![],
            body: Block {
                body: vec![],
                final_expr: None,
            },
        });

        let function = generate_function(ast);
        assert_eq!(
            function.chunk,
            Chunk {
                code: vec![Opcode::Null, Opcode::Return],
                constants: vec![],
            }
        )
    }
}
