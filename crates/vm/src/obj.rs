use std::collections::HashMap;

use bytecode::callables::Class;
use common::ProgramText;

use crate::{runtime_value::RuntimeValue, VM};

#[derive(Debug, Clone)]
pub struct ObjectInstance {
    pub class: Class,
    pub properties: HashMap<ProgramText, RuntimeValue>,
}

impl VM {
    pub(crate) fn new_obj(&mut self, class: Class) -> ObjectInstance {
        ObjectInstance {
            class,
            properties: HashMap::new(),
        }
    }
}
