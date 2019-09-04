use crate::structure::{FuncCall, ScopeId, VariableId};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Scope {
    symbols: HashMap<String, VariableId>,
    intermediates: Vec<VariableId>,
    body: Vec<FuncCall>,
    inputs: Vec<VariableId>,
    outputs: Vec<VariableId>,
    parent: Option<ScopeId>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            symbols: HashMap::new(),
            intermediates: Vec::new(),
            body: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            parent: Option::None,
        }
    }

    pub fn from_parent(parent: ScopeId) -> Scope {
        Scope {
            symbols: HashMap::new(),
            intermediates: Vec::new(),
            body: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            parent: Option::Some(parent),
        }
    }

    pub fn get_parent(&self) -> Option<ScopeId> {
        self.parent.clone()
    }

    pub fn add_func_call(&mut self, call: FuncCall) {
        self.body.push(call)
    }

    pub fn define_symbol(&mut self, symbol: &str, definition: VariableId) {
        self.symbols.insert(symbol.to_owned(), definition);
    }

    pub fn define_intermediate(&mut self, definition: VariableId) {
        self.intermediates.push(definition);
    }

    pub fn add_input(&mut self, input: VariableId) {
        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: VariableId) {
        self.outputs.push(output);
    }

    pub fn borrow_body(&self) -> &Vec<FuncCall> {
        &self.body
    }

    pub fn borrow_symbols(&self) -> &HashMap<String, VariableId> {
        &self.symbols
    }

    pub fn borrow_intermediates(&self) -> &Vec<VariableId> {
        &self.intermediates
    }

    pub fn get_input(&self, index: usize) -> VariableId {
        self.inputs[index]
    }

    pub fn borrow_inputs(&self) -> &Vec<VariableId> {
        &self.inputs
    }

    pub fn get_output(&self, index: usize) -> VariableId {
        self.outputs[index]
    }

    pub fn borrow_outputs(&self) -> &Vec<VariableId> {
        &self.outputs
    }

    pub fn get_single_output(&self) -> Option<VariableId> {
        if self.inputs.len() == 1 {
            Option::Some(self.inputs[0])
        } else {
            Option::None
        }
    }
}
