use crate::problem::FilePosition;
use crate::vague::structure::{
    DataType, KnownData, Program, Statement, VPExpression, Variable, VariableId,
};

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
DataType Auto;
DataType Bool;
DataType Int;
DataType Float;
DataType Void;
"#;

// Adds built-in methods to the root scope.
pub fn add_builtins(program: &mut Program) -> Builtins {
    let scope = program.get_entry_point();

    let automatic_type = program.adopt_and_define_symbol(
        scope,
        "Auto",
        Variable::data_type(FilePosition::placeholder(), DataType::Automatic),
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
        Variable::data_type(FilePosition::placeholder(), DataType::Float),
    );
    let void_type = program.adopt_and_define_symbol(
        scope,
        "Void",
        Variable::data_type(FilePosition::placeholder(), DataType::Void),
    );

    let data_type_literal = Box::new(VPExpression::Literal(
        KnownData::DataType(DataType::DataType),
        FilePosition::placeholder(),
    ));
    program[scope].add_statement(Statement::CreationPoint {
        var: automatic_type,
        var_type: data_type_literal.clone(),
        position: FilePosition::placeholder(),
    });
    program[scope].add_statement(Statement::CreationPoint {
        var: bool_type,
        var_type: data_type_literal.clone(),
        position: FilePosition::placeholder(),
    });
    program[scope].add_statement(Statement::CreationPoint {
        var: int_type,
        var_type: data_type_literal.clone(),
        position: FilePosition::placeholder(),
    });
    program[scope].add_statement(Statement::CreationPoint {
        var: float_type,
        var_type: data_type_literal.clone(),
        position: FilePosition::placeholder(),
    });
    program[scope].add_statement(Statement::CreationPoint {
        var: void_type,
        var_type: data_type_literal.clone(),
        position: FilePosition::placeholder(),
    });

    let builtins = Builtins {
        automatic_type,
        bool_type,
        int_type,
        float_type,
        void_type,
    };

    builtins
}
