use crate::{
    bytecode::{Address, BytecodeFrom, BytecodeGenerator, GenerationResult, Number, Opcode, Value},
    parser::expr::{Identifier, StructInitializer},
};

impl BytecodeFrom<StructInitializer> for BytecodeGenerator {
    fn generate(&mut self, struct_initializer: &StructInitializer) -> GenerationResult {
        let StructInitializer {
            identifier,
            properties,
        } = struct_initializer;

        for property in properties {
            self.add_constant(Value::String(property.name.clone()));
            self.generate(&property.expr)?;
        }

        // Opcode to tell class how many properties it should pop
        // It's doubled because we need to pop both key and value of the property
        // TODO: I'm not sure about handling these kind of stuff with f64, some other value type is needed for that
        self.add_constant(Value::Number((properties.len() * 2) as Number));

        // Find class address of which instance we would like to create
        self.generate(identifier)?;
        // Class instances are created by calling the class
        self.emit_code(Opcode::Call);

        Ok(())
    }
}
