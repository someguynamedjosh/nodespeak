use super::{Scope, Variable};

use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, PartialEq)]
pub struct ScopeId(usize);

impl Debug for ScopeId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "ss{}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct VariableId(usize);

impl Debug for VariableId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "sv{}", self.0)
    }
}

pub struct Program {
    scopes: Vec<Scope>,
    entry_point: ScopeId,
    variables: Vec<Variable>,
}