use crate::problem::FilePosition;
use crate::vague::structure::{DataType, Statement, Program, Variable, VariableId};

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
            DataType::Automatic,
        ),
    );
    let bool_type = program.adopt_and_define_symbol(
        scope,
        "Bool",
        Variable::data_type(FilePosition::placeholder(), DataType::Bool),
    );
    let int_type = program.adopt_and_define_symbol(
        scope,
        "Int",
        Variable::data_type(FilePosition::placeholder(), DataType::Int),
    );
    let float_type = program.adopt_and_define_symbol(
        scope,
        "Float",
        Variable::data_type(
            FilePosition::placeholder(),
            DataType::Float,
        ),
    );
    let void_type = program.adopt_and_define_symbol(
        scope,
        "Void",
        Variable::data_type(FilePosition::placeholder(), DataType::Void),
    );

    program[scope].add_statement(Statement::CreationPoint(
        automatic_type,
        FilePosition::placeholder(),
    ));
    program[scope].add_statement(Statement::CreationPoint(
        bool_type,
        FilePosition::placeholder(),
    ));
    program[scope].add_statement(Statement::CreationPoint(
        int_type,
        FilePosition::placeholder(),
    ));
    program[scope].add_statement(Statement::CreationPoint(
        float_type,
        FilePosition::placeholder(),
    ));
    program[scope].add_statement(Statement::CreationPoint(
        void_type,
        FilePosition::placeholder(),
    ));

    let builtins = Builtins {
        automatic_type,
        bool_type,
        int_type,
        float_type,
        void_type,
    };

    builtins
}
