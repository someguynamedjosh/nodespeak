use crate::vague::{ EntityId, ScopeId };

#[derive(Debug)]
pub struct VariableEntity {

}

impl VariableEntity {
  pub fn new() -> VariableEntity {
    VariableEntity { }
  }
}

#[derive(Debug)]
pub struct FunctionEntity {
  inputs: Vec<EntityId>,
  outputs: Vec<EntityId>,
  body: ScopeId,
}

impl FunctionEntity {
  pub fn new(body: ScopeId) -> FunctionEntity {
    FunctionEntity {
      inputs: Vec::new(),
      outputs: Vec::new(),
      body: body
    }
  }

  pub fn add_input(&mut self, input: EntityId) {
    self.inputs.push(input);
  }

  pub fn add_output(&mut self, output: EntityId) {
    self.outputs.push(output);
  }
}

#[derive(Debug)]
pub enum Entity {
  Variable(VariableEntity),
  Function(FunctionEntity),
}