use std::collections::HashMap;

use crate::{
    bytecode::{
        state::ScopeType,
        stmt::function::Function,
        value::{Callable, Value},
        BytecodeFrom, BytecodeGenerator, GenerationResult, Opcode,
    },
    parser::stmt::class::ClassStmt,
};

pub type Properties = HashMap<String, Value>;

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectInstance {
    // Temporary clone
    pub class: Class,
    pub properties: Properties,
}

impl Into<Value> for ObjectInstance {
    fn into(self) -> Value {
        Value::Object(self)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Function>,
    // pub superclass: Option<&'a Class<'a>>
    // pub properties: HashMap<String, Value>,
}

impl Class {
    pub fn new_instance(&self, args: Vec<Value>) -> ObjectInstance {
        let properties: Properties = args
            .chunks(2)
            .map(|values| {
                // TODO: This clone here probably can be avoided and we can operate directly on owned value

                let value = values.get(0).expect("Value needs to be present").clone();
                let key = values.get(1).expect("Key needs to be present").clone();
                (
                    key.into_string()
                        .expect("Somehow the key value is not a string"),
                    value,
                )
            })
            .collect();
        ObjectInstance {
            // TODO: Temporary class clone
            class: self.clone(),
            properties,
        }
    }
}

impl BytecodeFrom<ClassStmt> for BytecodeGenerator {
    fn generate(&mut self, class: &ClassStmt) -> GenerationResult {
        let ClassStmt {
            name,
            property_initializers,
            methods,
            superclass,
        } = class;
        self.enter_callable(ScopeType::Class);

        // let properties = property_initializers.

        let class = Class {
            name: name.clone(),
            methods: vec![],
            // methods: methods.iter().map(|),
            // properties: HashMap::new(),
        };

        self.fn_chunks.pop();

        self.state.leave_scope();
        self.add_constant(Value::Callable(Callable::Class(class)));
        self.state.declare_var(name);

        Ok(())
    }
}
