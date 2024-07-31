use std::collections::HashMap;
use crate::interpreter::Value;
use crate::interpreter::RuntimeError;


#[derive(Default, Clone)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Option<Value>>
}

impl Environment {
    pub fn enclosing(&mut self, enclosing: Option<Box<Environment>>) {
        self.enclosing = enclosing;
    }

    pub fn get_enclosing(&mut self) -> Option<Box<Environment>> {
        self.enclosing.clone()
    }

    pub fn define(&mut self, name: String, value: Option<Value>) {
        self.values.entry(name).and_modify(|v| *v = value.clone()).or_insert(value.clone());
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), RuntimeError>{
        return match self.values.get_mut(&name) {
            Some(v) => {*v = Some(value); Ok(())},
            None => {
                if let Some(ref mut enclosing) = self.enclosing {
                    enclosing.assign(name, value)
                } else {
                    Err(RuntimeError::VariableNotFound)
                }
            }
        }
    }

    pub fn get(&self, name: String) -> Result<Value, RuntimeError> {
        match self.values.get(&name) {
            Some(v) => {match v {
                Some(v2) => Ok(v2.clone()),
                None => Err(RuntimeError::VariableNotInitialized)
            }},
            None => {
                if let Some(ref enclosing) = self.enclosing {
                    enclosing.get(name)
                } else {
                    Err(RuntimeError::VariableNotFound)
                }
            }
        }
    }
}
