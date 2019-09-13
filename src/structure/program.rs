use crate::interpreter::{self, InterpreterOutcome};
use crate::problem::{self, CompileProblem, FilePosition};
use crate::structure::{
    add_builtins, Builtins, DataType, Expression, FunctionData, KnownData, Scope, Variable,
};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};

/// Refers to a [`Scope`] stored in a [`Program`].
///
/// You'll notice that this struct requires no lifetime. This was chosen to allow for easy
/// implementation of tree-like and cyclic data structures inside the library.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ScopeId(usize);

impl Debug for ScopeId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "s{}", self.0)
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct VariableId(usize);

impl Debug for VariableId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "v{}", self.0)
    }
}

/// Represents an entire program written in the Waveguide language.
pub struct Program {
    scopes: Vec<Scope>,
    entry_point: ScopeId,
    variables: Vec<Variable>,
    builtins: Option<Builtins>,
    automatic_interpretation: bool,
}

impl Debug for Program {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "entry point: {:?}", self.entry_point)?;
        for (index, scope) in self.scopes.iter().enumerate() {
            write!(formatter, "\ncontents of {:?}:\n", ScopeId(index))?;
            write!(
                formatter,
                "    {}",
                format!("{:?}", scope).replace("\n", "\n    ")
            )?;
        }
        for (index, variable) in self.variables.iter().enumerate() {
            write!(formatter, "\ndetails for {:?}:\n", VariableId(index))?;
            write!(
                formatter,
                "    {}",
                format!("{:?}", variable).replace("\n", "\n    ")
            )?;
        }
        write!(formatter, "")
    }
}

impl Program {
    pub fn new() -> Program {
        let mut prog = Program {
            scopes: vec![Scope::new()],
            entry_point: ScopeId(0),
            variables: Vec::new(),
            builtins: Option::None,
            automatic_interpretation: false,
        };
        prog.builtins = Option::Some(add_builtins(&mut prog));
        prog
    }

    // ===SCOPES====================================================================================

    /// Creates a new scope that has no parent.
    pub fn create_scope(&mut self) -> ScopeId {
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Scope::new());
        id
    }

    pub fn create_child_scope(&mut self, parent: ScopeId) -> ScopeId {
        assert!(parent.0 < self.scopes.len());
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Scope::from_parent(parent));
        id
    }

    pub fn borrow_scope(&self, scope: ScopeId) -> &Scope {
        assert!(scope.0 < self.scopes.len());
        &self.scopes[scope.0]
    }

    pub fn borrow_scope_mut(&mut self, scope: ScopeId) -> &mut Scope {
        assert!(scope.0 < self.scopes.len());
        &mut self.scopes[scope.0]
    }

    pub fn add_expression(
        &mut self,
        add_to: ScopeId,
        expression: Expression,
    ) -> Result<(), CompileProblem> {
        assert!(add_to.0 < self.scopes.len());
        if self.automatic_interpretation {
            match interpreter::interpret_expression(self, &expression, false) {
                InterpreterOutcome::UnknownData | InterpreterOutcome::UnknownFunction => {
                    self.scopes[add_to.0].add_expression(expression)
                }
                InterpreterOutcome::AssertFailed(location) => {
                    return Result::Err(problem::guaranteed_assert(location))
                }
                InterpreterOutcome::Problem(problem) => return Result::Err(problem),
                // TODO? check that data is Void
                InterpreterOutcome::Specific(..) | InterpreterOutcome::Returned => (),
            }
        } else {
            self.scopes[add_to.0].add_expression(expression);
        }
        Result::Ok(())
    }

    pub fn clone_scope_body(&self, scope: ScopeId) -> Vec<Expression> {
        self.borrow_scope(scope).borrow_body().clone()
    }

    pub fn clone_symbols_in(&self, scope: ScopeId) -> HashMap<String, VariableId> {
        self.borrow_scope(scope).borrow_symbols().clone()
    }

    pub fn clone_intermediates_in(&self, scope: ScopeId) -> Vec<VariableId> {
        self.borrow_scope(scope).borrow_intermediates().clone()
    }

    // ===SYMBOLS/VARIABLES=========================================================================

    pub fn shallow_lookup_symbol(&self, scope: ScopeId, symbol: &str) -> Option<VariableId> {
        match self.borrow_scope(scope).borrow_symbols().get(symbol) {
            Option::Some(value) => Option::Some(*value),
            Option::None => Option::None,
        }
    }

    pub fn lookup_symbol(&self, scope: ScopeId, symbol: &str) -> Option<VariableId> {
        match self.borrow_scope(scope).borrow_symbols().get(symbol) {
            Option::Some(value_) => Option::Some(*value_),
            Option::None => match self.borrow_scope(scope).get_parent() {
                Option::Some(parent) => self.lookup_symbol(parent, symbol),
                Option::None => Option::None,
            },
        }
    }

    pub fn modify_variable(&mut self, variable: VariableId, modified: Variable) {
        assert!(variable.0 < self.variables.len());
        self.variables[variable.0] = modified;
    }

    pub fn adopt_variable(&mut self, variable: Variable) -> VariableId {
        let id = VariableId(self.variables.len());
        self.variables.push(variable);
        id
    }

    pub fn borrow_variable(&self, variable: VariableId) -> &Variable {
        assert!(variable.0 < self.variables.len());
        &self.variables[variable.0]
    }

    pub fn borrow_variable_mut(&mut self, variable: VariableId) -> &mut Variable {
        assert!(variable.0 < self.variables.len());
        &mut self.variables[variable.0]
    }

    pub fn set_data_type(&mut self, variable: VariableId, data_type: DataType) {
        assert!(variable.0 < self.variables.len());
        self.variables[variable.0].set_data_type(data_type);
    }

    // ===DATA STORAGE==============================================================================

    /// Sets the value that the specified variable is known to have at the start of the program.
    ///
    /// By default, all variables are given appropriate default values for their value types. Literal
    /// variables such as [`Variable::IntLiteral`] are set to whatever the value of the literal is.
    pub fn set_initial_value(&mut self, variable: VariableId, value: KnownData) {
        assert!(variable.0 < self.variables.len());
        self.variables[variable.0].set_initial_value(value);
    }

    /// Gets the value that the specified variable is known to have at the start of the program.
    ///
    /// By default, all variables are given appropriate default values for their value types. Literal
    /// variables such as [`Variable::IntLiteral`] are set to whatever the value of the literal is.
    pub fn borrow_initial_value(&self, variable: VariableId) -> &KnownData {
        assert!(variable.0 < self.variables.len());
        &self.variables[variable.0].borrow_initial_value()
    }

    /// Marks the specified variable as having a value that remains unchanged from its initial value
    /// throughout the entire program.
    pub fn mark_as_permanent(&mut self, variable: VariableId) {
        assert!(variable.0 < self.variables.len());
        self.variables[variable.0].mark_as_permanent();
    }

    /// Checks if the value of the specified variable remains unchanged from the initial value
    /// throughout the entire program.
    pub fn is_permanent(&mut self, variable: VariableId) -> bool {
        assert!(variable.0 < self.variables.len());
        self.variables[variable.0].is_permanent()
    }

    /// Stores a temporary value for the related variable.
    ///
    /// This is used during interpretation and simplification to keep track of the current value
    /// of a variable that does not have a permanent value across the entire program.
    pub fn set_temporary_value(&mut self, variable: VariableId, value: KnownData) {
        assert!(variable.0 < self.variables.len());
        self.variables[variable.0].set_temporary_value(value);
    }

    pub fn set_value_of(&mut self, expression: &Expression, value: KnownData) {
        (*self.borrow_value_of_mut(expression)) = value;
    }

    /// Retrieves a temporary value for the related variable, set by set_temporary_value.
    ///
    /// This is used during interpretation and simplification to keep track of the current value
    /// of a variable that does not have a permanent value across the entire program.
    pub fn borrow_temporary_value(&self, variable: VariableId) -> &KnownData {
        assert!(variable.0 < self.variables.len());
        self.variables[variable.0].borrow_temporary_value()
    }

    pub fn borrow_temporary_value_mut(&mut self, variable: VariableId) -> &mut KnownData {
        assert!(variable.0 < self.variables.len());
        self.variables[variable.0].borrow_temporary_value_mut()
    }

    pub fn borrow_value_of<'a>(&'a self, expression: &'a Expression) -> &'a KnownData {
        match expression {
            Expression::Access { base, indexes } => {
                // TODO handle expressions that need to be interpreted before we can determine
                // their value.
                let mut real_indexes = Vec::new();
                for index in indexes {
                    match self.borrow_value_of(index) {
                        // TODO check that value is positive.
                        KnownData::Int(value) => real_indexes.push(*value as usize),
                        _ => panic!("TODO error, indexes must be known integers."),
                    }
                }
                let data = self.borrow_value_of(base.borrow());
                if let KnownData::Array(contents) = data {
                    if !contents.is_inside(&real_indexes) {
                        panic!("TODO error, indexes must be inside array bounds.")
                    }
                    contents.borrow_item(&real_indexes)
                } else {
                    panic!("TODO error, base of access expression is not array.")
                }
            }
            Expression::Literal(data) => data,
            Expression::Variable(id) => self.borrow_temporary_value(*id),
            _ => panic!("TODO: error, cannot borrow value of complex expression."),
        }
    }

    pub fn borrow_value_of_mut(&mut self, expression: &Expression) -> &mut KnownData {
        match expression {
            Expression::Access { base, indexes } => {
                // TODO handle expressions that need to be interpreted before we can determine
                // their value.
                let mut real_indexes = Vec::new();
                for index in indexes {
                    match self.borrow_value_of(index) {
                        // TODO check that value is positive.
                        KnownData::Int(value) => real_indexes.push(*value as usize),
                        _ => panic!("TODO error, indexes must be known integers."),
                    }
                }
                let data = self.borrow_value_of_mut(base.borrow());
                if let KnownData::Array(contents) = data {
                    if !contents.is_inside(&real_indexes) {
                        panic!("TODO error, indexes must be inside array bounds.")
                    }
                    contents.borrow_item_mut(&real_indexes)
                } else {
                    panic!("TODO error, base of access expression is not array.")
                }
            }
            Expression::Literal(..) => panic!("TODO error, cannot borrow literal as mutable."),
            Expression::Variable(id) => self.borrow_temporary_value_mut(*id),
            _ => panic!("TODO: error, cannot borrow value of complex expression."),
        }
    }

    /// Resets the temporary value of an variable to be the initial value of that variable.
    pub fn reset_temporary_value(&mut self, variable: VariableId) {
        assert!(variable.0 < self.variables.len());
        self.variables[variable.0].reset_temporary_value();
    }

    /// Resets the temporary value of every variable to their permanent value values.
    ///
    /// This should be used before beginning interpretation or simplification to reset all values
    /// to a value they are known to have at the beginning of the program.
    pub fn reset_all_temporary_value(&mut self) {
        for variable in self.variables.iter_mut() {
            variable.reset_temporary_value();
        }
    }

    /// Checks if the variable had multiple values read from it at any point during the program.
    ///
    /// Only use this after the entirety of a program has been interpreted.
    // pub fn had_multiple_temporary_values(&self, variable: VariableId) -> bool {
    //     assert!(variable.0 < self.variables.len());
    //     self.variables[variable.0].had_multiple_temporary_values()
    // }

    // ===INTERPRETATION============================================================================
    pub fn enable_automatic_interpretation(&mut self) {
        self.automatic_interpretation = true;
    }

    pub fn disable_automatic_interpretation(&mut self) {
        self.automatic_interpretation = false;
    }

    // ===UTILITIES=================================================================================

    pub fn get_entry_point(&self) -> ScopeId {
        self.entry_point
    }

    pub fn set_entry_point(&mut self, new_entry_point: ScopeId) {
        self.entry_point = new_entry_point;
    }

    pub fn get_builtins(&self) -> &Builtins {
        self.builtins.as_ref().unwrap()
    }

    pub fn adopt_and_define_symbol(
        &mut self,
        scope: ScopeId,
        symbol: &str,
        definition: Variable,
    ) -> VariableId {
        let id = self.adopt_variable(definition);
        self.borrow_scope_mut(scope).define_symbol(symbol, id);
        id
    }

    pub fn adopt_and_define_intermediate(
        &mut self,
        scope: ScopeId,
        definition: Variable,
    ) -> VariableId {
        let id = self.adopt_variable(definition);
        self.borrow_scope_mut(scope).define_intermediate(id);
        id
    }

    pub fn make_intermediate_auto_var(
        &mut self,
        scope: ScopeId,
        position: FilePosition,
    ) -> VariableId {
        self.adopt_and_define_intermediate(scope, Variable::automatic(position))
    }

    // Tries to find a function variable which uses the specified scope as a
    // function body. If nothing is found, Option::None is returned. Note that
    // while the data in the variable is guaranteed to contain correct
    // information about e.g. the inputs and outputs of the function, it is not
    // guaranteed to be the actual original variable that described the function
    // when the program was first parsed.
    pub fn find_parent_function(&self, scope: ScopeId) -> Option<VariableId> {
        // TODO: This is very inefficient.
        // TODO: This function might be useless and buggy, need to review how it
        // is used in the rest of the code.
        let mut real_scope = scope.0;
        loop {
            let mut index: usize = 0;
            for variable in self.variables.iter() {
                if variable.is_permanent() {
                    if let KnownData::Function(data) = variable.borrow_initial_value() {
                        if data.get_body() == scope {
                            return Option::Some(VariableId(index));
                        }
                    }
                }
                index += 1;
            }
            match self.scopes[real_scope].get_parent() {
                Option::None => return Option::None,
                Option::Some(id) => real_scope = id.0,
            };
        }
    }

    // Tries to find a function variable which uses the specified scope as a
    // function body. If nothing is found, Option::None is returned.
    pub fn lookup_and_clone_parent_function(&self, scope: ScopeId) -> Option<FunctionData> {
        // TODO: This is very inefficient.
        // TODO: This function might be useless and buggy, need to review how it
        // is used in the rest of the code.
        let mut real_scope = scope.0;
        loop {
            for variable in self.variables.iter() {
                if variable.is_permanent() {
                    if let KnownData::Function(data) = variable.borrow_initial_value() {
                        if data.get_body() == scope {
                            return Option::Some(data.clone());
                        }
                    }
                }
            }
            match self.scopes[real_scope].get_parent() {
                Option::None => return Option::None,
                Option::Some(id) => real_scope = id.0,
            };
        }
    }
}
