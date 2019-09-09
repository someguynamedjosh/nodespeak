use crate::problem::FilePosition;
use crate::structure::{BaseType, Program, Variable, VariableId};

#[readonly::make]
#[derive(Debug)]
pub struct Builtins {
    pub automatic_type: VariableId,
    pub bool_type: VariableId,
    pub int_type: VariableId,
    pub float_type: VariableId,
    pub void_type: VariableId,
}

pub const FAKE_BUILTIN_SOURCE: &str = r#"
DataType_ Auto;
DataType_ Bool;
DataType_ Int;
DataType_ Float;
DataType_ Void;
"#;

// Adds built-in methods to the root scope.
pub fn add_builtins(program: &mut Program) -> Builtins {
    let scope = program.get_entry_point();

    let automatic_type = program.adopt_and_define_symbol(
        scope,
        "Auto",
        Variable::data_type(
            FilePosition::placeholder(),
            BaseType::Automatic.to_scalar_type(),
        ),
    );
    let bool_type = program.adopt_and_define_symbol(
        scope,
        "Bool",
        Variable::data_type(FilePosition::placeholder(), BaseType::Bool.to_scalar_type()),
    );
    let int_type = program.adopt_and_define_symbol(
        scope,
        "Int",
        Variable::data_type(FilePosition::placeholder(), BaseType::Int.to_scalar_type()),
    );
    let float_type = program.adopt_and_define_symbol(
        scope,
        "Float",
        Variable::data_type(
            FilePosition::placeholder(),
            BaseType::Float.to_scalar_type(),
        ),
    );
    let void_type = program.adopt_and_define_symbol(
        scope,
        "Void",
        Variable::data_type(FilePosition::placeholder(), BaseType::Void.to_scalar_type()),
    );

    let builtins = Builtins {
        automatic_type,
        bool_type,
        int_type,
        float_type,
        void_type,
    };

    builtins
}
