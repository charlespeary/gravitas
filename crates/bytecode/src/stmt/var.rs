#[cfg(test)]
mod test {
    use parser::parse::expr::atom::AtomicValue;

    use crate::{
        chunk::Constant,
        test::{declare_var, expr, expr_stmt},
        BytecodeFrom, BytecodeGenerator, MemoryAddress,
    };

    #[test]
    fn finds_local_variable() {
        let mut generator = BytecodeGenerator::new();
        let data = vec![
            declare_var("local".to_owned(), expr(AtomicValue::Number(0.0))),
            expr_stmt(expr(AtomicValue::Identifier {
                properties: vec![],
                is_assignment: false,
                name: "local".to_owned(),
            })),
        ];

        generator
            .generate(data)
            .expect("Failed to generate bytecode which finds local variable.");

        let bytecode = generator.code().chunk;
        assert_eq!(
            bytecode.constants[1],
            Constant::MemoryAddress(MemoryAddress::Local(0))
        )
    }

    fn finds_variable_in_upper_scope() {}

    fn finds_global_variable() {}

    fn finds_closed_variable() {}
}
