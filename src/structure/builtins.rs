use crate::problem::FilePosition;
use crate::structure::{DataType, FunctionData, Program, Variable, VariableId};

#[derive(Clone, Debug)]
pub enum BuiltinFunction {
    Add,
    Subtract,
    Multiply,
    Divide,
    IntDiv,
    Reciprocal,
    Modulo,
    Power,
    BAnd,
    BOr,
    BXor,
    BNot,
    IntToFloat,
    BoolToFloat,
    BoolToInt,
    IntToBool,
    FloatToInt,
    FloatToBool,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    And,
    Or,
    Xor,
    Not,
    Assert,
    Copy,
    Return,
}

#[readonly::make]
#[derive(Debug)]
pub struct Builtins {
    pub add_func: VariableId,
    pub sub_func: VariableId,
    pub mul_func: VariableId,
    pub div_func: VariableId,
    pub int_div_func: VariableId,
    pub recip_func: VariableId,
    pub mod_func: VariableId,
    pub pow_func: VariableId,

    pub band_func: VariableId,
    pub bor_func: VariableId,
    pub bxor_func: VariableId,
    pub bnot_func: VariableId,

    pub int_to_float_func: VariableId,
    pub bool_to_float_func: VariableId,
    pub bool_to_int_func: VariableId,
    pub int_to_bool_func: VariableId,
    pub float_to_int_func: VariableId,
    pub float_to_bool_func: VariableId,

    pub eq_func: VariableId,
    pub neq_func: VariableId,
    pub lte_func: VariableId,
    pub gte_func: VariableId,
    pub lt_func: VariableId,
    pub gt_func: VariableId,

    pub and_func: VariableId,
    pub or_func: VariableId,
    pub xor_func: VariableId,
    pub not_func: VariableId,

    pub assert_func: VariableId,
    pub copy_func: VariableId,
    pub return_func: VariableId,

    pub automatic_type: VariableId,
    pub bool_type: VariableId,
    pub int_type: VariableId,
    pub float_type: VariableId,
    pub void_type: VariableId,
}

pub const FAKE_BUILTIN_SOURCE: &str = r#"
builtin fn add(T? in1, T? in2):T;
builtin fn subtract(T? in1, T? in2):T;
builtin fn multiply(T? in1, T? in2):T;
builtin fn divide(Float in1, Float in2):Float;
builtin fn int_div(Int in1, Int in2):Int;
builtin fn modulo(T? in1, T? in2):T;
builtin fn power(T? in1, T? in2):T;

DataType_ Auto;
DataType_ Bool;
DataType_ Int;
DataType_ Float;
DataType_ Void;
"#;

// Adds built-in methods to the root scope.
pub fn add_builtins(program: &mut Program) -> Builtins {
    let scope = program.get_root_scope();

    let automatic_type =
        program.adopt_and_define_symbol(scope, "Auto", Variable::data_type(DataType::Automatic));
    let bool_type =
        program.adopt_and_define_symbol(scope, "Bool", Variable::data_type(DataType::Bool));
    let int_type =
        program.adopt_and_define_symbol(scope, "Int", Variable::data_type(DataType::Int));
    let float_type =
        program.adopt_and_define_symbol(scope, "Float", Variable::data_type(DataType::Float));
    let void_type =
        program.adopt_and_define_symbol(scope, "Void", Variable::data_type(DataType::Void));

    let make_blank_func = |program: &mut Program,
                           func: BuiltinFunction,
                           name: &str,
                           header_start: usize,
                           header_end: usize| {
        let func_scope = program.create_scope();
        let func_data = FunctionData::builtin(
            func_scope,
            func,
            FilePosition::for_builtin(header_start, header_end),
        );
        program.adopt_and_define_symbol(scope, name, Variable::function_def(func_data))
    };

    let make_c_func = |program: &mut Program,
                       func: BuiltinFunction,
                       name: &str,
                       in_type,
                       header_start: usize,
                       header_end: usize| {
        let func_scope = program.create_scope();
        let mut func_data = FunctionData::builtin(
            func_scope,
            func,
            FilePosition::for_builtin(header_start, header_end),
        );
        let input =
            program.adopt_and_define_symbol(func_scope, "input", Variable::variable(in_type, None));
        func_data.add_input(input);
        program.adopt_and_define_symbol(scope, name, Variable::function_def(func_data))
    };

    let make_a_a_func = |program: &mut Program,
                         func: BuiltinFunction,
                         name: &str,
                         header_start: usize,
                         header_end: usize| {
        let func_scope = program.create_scope();
        let mut func_data = FunctionData::builtin(
            func_scope,
            func,
            FilePosition::for_builtin(header_start, header_end),
        );
        let parameter = program.adopt_and_define_symbol(
            func_scope,
            "TYPE",
            Variable::variable(DataType::DataType_, None),
        );
        let data_type = DataType::LoadTemplateParameter(parameter);
        let in1 = program.adopt_and_define_symbol(
            func_scope,
            "in1",
            Variable::variable(data_type.clone(), None),
        );
        let out =
            program.adopt_and_define_symbol(func_scope, "out", Variable::variable(data_type, None));
        func_data.add_input(in1);
        func_data.add_output(out);
        program.adopt_and_define_symbol(scope, name, Variable::function_def(func_data))
    };

    let make_aa_a_func = |program: &mut Program,
                          func: BuiltinFunction,
                          name: &str,
                          header_start: usize,
                          header_end: usize| {
        let func_scope = program.create_scope();
        let mut func_data = FunctionData::builtin(
            func_scope,
            func,
            FilePosition::for_builtin(header_start, header_end),
        );
        let parameter = program.adopt_and_define_symbol(
            func_scope,
            "TYPE",
            Variable::variable(DataType::DataType_, None),
        );
        let data_type = DataType::LoadTemplateParameter(parameter);
        let in1 = program.adopt_and_define_symbol(
            func_scope,
            "in1",
            Variable::variable(data_type.clone(), None),
        );
        let in2 = program.adopt_and_define_symbol(
            func_scope,
            "in2",
            Variable::variable(data_type.clone(), None),
        );
        let out =
            program.adopt_and_define_symbol(func_scope, "out", Variable::variable(data_type, None));
        func_data.add_input(in1);
        func_data.add_input(in2);
        func_data.add_output(out);
        program.adopt_and_define_symbol(scope, name, Variable::function_def(func_data))
    };

    let make_logic_func = |program: &mut Program,
                           func: BuiltinFunction,
                           name: &str,
                           header_start: usize,
                           header_end: usize| {
        let func_scope = program.create_scope();
        let mut func_data = FunctionData::builtin(
            func_scope,
            func,
            FilePosition::for_builtin(header_start, header_end),
        );
        let in1 = program.adopt_and_define_symbol(
            func_scope,
            "in1",
            Variable::variable(DataType::Bool, None),
        );
        let in2 = program.adopt_and_define_symbol(
            func_scope,
            "in2",
            Variable::variable(DataType::Bool, None),
        );
        let out = program.adopt_and_define_symbol(
            func_scope,
            "out",
            Variable::variable(DataType::Bool, None),
        );
        func_data.add_input(in1);
        func_data.add_input(in2);
        func_data.add_output(out);
        program.adopt_and_define_symbol(scope, name, Variable::function_def(func_data))
    };

    let make_convert_func = |program: &mut Program,
                             func: BuiltinFunction,
                             name: &str,
                             in_type,
                             out_type,
                             header_start: usize,
                             header_end: usize| {
        let func_scope = program.create_scope();
        let mut func_data = FunctionData::builtin(
            func_scope,
            func,
            FilePosition::for_builtin(header_start, header_end),
        );
        let in1 =
            program.adopt_and_define_symbol(func_scope, "in1", Variable::variable(in_type, None));
        let out =
            program.adopt_and_define_symbol(func_scope, "out", Variable::variable(out_type, None));
        func_data.add_input(in1);
        func_data.add_output(out);
        program.adopt_and_define_symbol(scope, name, Variable::function_def(func_data))
    };

    let make_comparison_func = |program: &mut Program,
                                func: BuiltinFunction,
                                name: &str,
                                header_start: usize,
                                header_end: usize| {
        let func_scope = program.create_scope();
        let mut func_data = FunctionData::builtin(
            func_scope,
            func,
            FilePosition::for_builtin(header_start, header_end),
        );
        let parameter = program.adopt_and_define_symbol(
            func_scope,
            "TYPE",
            Variable::variable(DataType::DataType_, None),
        );
        let data_type = DataType::LoadTemplateParameter(parameter);
        let in1 = program.adopt_and_define_symbol(
            func_scope,
            "in1",
            Variable::variable(data_type.clone(), None),
        );
        let in2 =
            program.adopt_and_define_symbol(func_scope, "in2", Variable::variable(data_type, None));
        let out = program.adopt_and_define_symbol(
            func_scope,
            "out",
            Variable::variable(DataType::Bool, None),
        );
        func_data.add_input(in1);
        func_data.add_input(in2);
        func_data.add_output(out);
        program.adopt_and_define_symbol(scope, name, Variable::function_def(func_data))
    };

    let builtins = Builtins {
        add_func: make_aa_a_func(program, BuiltinFunction::Add, "!add", 1, 34),
        sub_func: make_aa_a_func(program, BuiltinFunction::Subtract, "!subtract", 35, 73),

        mul_func: make_aa_a_func(program, BuiltinFunction::Multiply, "!multiply", 74, 112),
        div_func: make_aa_a_func(program, BuiltinFunction::Divide, "!divide", 0, 0),
        int_div_func: make_aa_a_func(program, BuiltinFunction::IntDiv, "!int_div", 0, 0),
        mod_func: make_aa_a_func(program, BuiltinFunction::Modulo, "!modulo", 0, 0),
        pow_func: make_aa_a_func(program, BuiltinFunction::Power, "!power", 0, 0),
        recip_func: make_aa_a_func(program, BuiltinFunction::Reciprocal, "!reciprocal", 0, 0),

        band_func: make_aa_a_func(program, BuiltinFunction::BAnd, "!band", 0, 0),
        bor_func: make_aa_a_func(program, BuiltinFunction::BOr, "!bor", 0, 0),
        bxor_func: make_aa_a_func(program, BuiltinFunction::BXor, "!bxor", 0, 0),
        bnot_func: make_aa_a_func(program, BuiltinFunction::BNot, "!bnot", 0, 0),

        int_to_float_func: make_convert_func(
            program,
            BuiltinFunction::IntToFloat,
            "!int_to_float",
            DataType::Int,
            DataType::Float,
            0,
            0,
        ),
        float_to_int_func: make_convert_func(
            program,
            BuiltinFunction::FloatToInt,
            "!float_to_int",
            DataType::Float,
            DataType::Int,
            0,
            0,
        ),
        bool_to_float_func: make_convert_func(
            program,
            BuiltinFunction::BoolToFloat,
            "!bool_to_float",
            DataType::Bool,
            DataType::Float,
            0,
            0,
        ),
        float_to_bool_func: make_convert_func(
            program,
            BuiltinFunction::FloatToBool,
            "!float_to_bool",
            DataType::Float,
            DataType::Bool,
            0,
            0,
        ),
        bool_to_int_func: make_convert_func(
            program,
            BuiltinFunction::BoolToInt,
            "!bool_to_int",
            DataType::Bool,
            DataType::Int,
            0,
            0,
        ),
        int_to_bool_func: make_convert_func(
            program,
            BuiltinFunction::IntToBool,
            "!int_to_bool",
            DataType::Int,
            DataType::Bool,
            0,
            0,
        ),

        eq_func: make_comparison_func(program, BuiltinFunction::Equal, "!equal", 0, 0),
        neq_func: make_comparison_func(program, BuiltinFunction::NotEqual, "!not_equal", 0, 0),
        lte_func: make_comparison_func(
            program,
            BuiltinFunction::LessThanOrEqual,
            "!less_than_or_equal",
            0,
            0,
        ),
        gte_func: make_comparison_func(
            program,
            BuiltinFunction::GreaterThanOrEqual,
            "!greater_than_or_equal",
            0,
            0,
        ),
        lt_func: make_comparison_func(program, BuiltinFunction::LessThan, "!less_than", 0, 0),
        gt_func: make_comparison_func(program, BuiltinFunction::GreaterThan, "!greater_than", 0, 0),

        and_func: make_logic_func(program, BuiltinFunction::And, "!and", 0, 0),
        or_func: make_logic_func(program, BuiltinFunction::Or, "!or", 0, 0),
        xor_func: make_logic_func(program, BuiltinFunction::Xor, "!xor", 0, 0),
        not_func: make_logic_func(program, BuiltinFunction::Not, "!not", 0, 0),

        assert_func: make_c_func(
            program,
            BuiltinFunction::Assert,
            "assert",
            DataType::Bool,
            0,
            0,
        ),
        copy_func: make_a_a_func(program, BuiltinFunction::Copy, "!copy", 0, 0),
        return_func: make_blank_func(program, BuiltinFunction::Return, "!return", 0, 0),

        automatic_type: automatic_type,
        bool_type: bool_type,
        int_type: int_type,
        float_type: float_type,
        void_type: void_type,
    };

    builtins
}
