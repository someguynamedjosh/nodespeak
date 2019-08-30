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
    
    let make_blank_func = |program: &mut Program, func: BuiltinFunction, name: &str| {
        let func_scope = program.create_scope();
        let func_data = FunctionData::builtin(func_scope, func);
        program.adopt_and_define_symbol(scope, name, Variable::function_def(func_data))
    };

    let make_c_func = |program: &mut Program, func: BuiltinFunction, name: &str, in_type| {
        let func_scope = program.create_scope();
        let mut func_data = FunctionData::builtin(func_scope, func);
        let input =
            program.adopt_and_define_symbol(func_scope, "input", Variable::variable(in_type, None));
        func_data.add_input(input);
        program.adopt_and_define_symbol(scope, name, Variable::function_def(func_data))
    };

    let make_a_a_func = |program: &mut Program, func: BuiltinFunction, name: &str| {
        let func_scope = program.create_scope();
        let mut func_data = FunctionData::builtin(func_scope, func);
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

    let make_aa_a_func = |program: &mut Program, func: BuiltinFunction, name: &str| {
        let func_scope = program.create_scope();
        let mut func_data = FunctionData::builtin(func_scope, func);
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

    let make_logic_func = |program: &mut Program, func: BuiltinFunction, name: &str| {
        let func_scope = program.create_scope();
        let mut func_data = FunctionData::builtin(func_scope, func);
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
                             out_type| {
        let func_scope = program.create_scope();
        let mut func_data = FunctionData::builtin(func_scope, func);
        let in1 =
            program.adopt_and_define_symbol(func_scope, "in1", Variable::variable(in_type, None));
        let out =
            program.adopt_and_define_symbol(func_scope, "out", Variable::variable(out_type, None));
        func_data.add_input(in1);
        func_data.add_output(out);
        program.adopt_and_define_symbol(scope, name, Variable::function_def(func_data))
    };

    let make_comparison_func = |program: &mut Program, func: BuiltinFunction, name: &str| {
        let func_scope = program.create_scope();
        let mut func_data = FunctionData::builtin(func_scope, func);
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
        add_func: make_aa_a_func(program, BuiltinFunction::Add, "!add"),
        sub_func: make_aa_a_func(program, BuiltinFunction::Subtract, "!subtract"),
        mul_func: make_aa_a_func(program, BuiltinFunction::Multiply, "!multiply"),
        div_func: make_aa_a_func(program, BuiltinFunction::Divide, "!divide"),
        int_div_func: make_aa_a_func(program, BuiltinFunction::IntDiv, "!int_div"),
        mod_func: make_aa_a_func(program, BuiltinFunction::Modulo, "!modulo"),
        pow_func: make_aa_a_func(program, BuiltinFunction::Power, "!power"),
        recip_func: make_aa_a_func(program, BuiltinFunction::Reciprocal, "!reciprocal"),

        band_func: make_aa_a_func(program, BuiltinFunction::BAnd, "!band"),
        bor_func: make_aa_a_func(program, BuiltinFunction::BOr, "!bor"),
        bxor_func: make_aa_a_func(program, BuiltinFunction::BXor, "!bxor"),
        bnot_func: make_aa_a_func(program, BuiltinFunction::BNot, "!bnot"),

        int_to_float_func: make_convert_func(
            program,
            BuiltinFunction::IntToFloat,
            "!int_to_float",
            DataType::Int,
            DataType::Float,
        ),
        float_to_int_func: make_convert_func(
            program,
            BuiltinFunction::FloatToInt,
            "!float_to_int",
            DataType::Float,
            DataType::Int,
        ),
        bool_to_float_func: make_convert_func(
            program,
            BuiltinFunction::BoolToFloat,
            "!bool_to_float",
            DataType::Bool,
            DataType::Float,
        ),
        float_to_bool_func: make_convert_func(
            program,
            BuiltinFunction::FloatToBool,
            "!float_to_bool",
            DataType::Float,
            DataType::Bool,
        ),
        bool_to_int_func: make_convert_func(
            program,
            BuiltinFunction::BoolToInt,
            "!bool_to_int",
            DataType::Bool,
            DataType::Int,
        ),
        int_to_bool_func: make_convert_func(
            program,
            BuiltinFunction::IntToBool,
            "!int_to_bool",
            DataType::Int,
            DataType::Bool,
        ),

        eq_func: make_comparison_func(program, BuiltinFunction::Equal, "!equal"),
        neq_func: make_comparison_func(program, BuiltinFunction::NotEqual, "!not_equal"),
        lte_func: make_comparison_func(
            program,
            BuiltinFunction::LessThanOrEqual,
            "!less_than_or_equal",
        ),
        gte_func: make_comparison_func(
            program,
            BuiltinFunction::GreaterThanOrEqual,
            "!greater_than_or_equal",
        ),
        lt_func: make_comparison_func(program, BuiltinFunction::LessThan, "!less_than"),
        gt_func: make_comparison_func(program, BuiltinFunction::GreaterThan, "!greater_than"),

        and_func: make_logic_func(program, BuiltinFunction::And, "!and"),
        or_func: make_logic_func(program, BuiltinFunction::Or, "!or"),
        xor_func: make_logic_func(program, BuiltinFunction::Xor, "!xor"),
        not_func: make_logic_func(program, BuiltinFunction::Not, "!not"),

        assert_func: make_c_func(program, BuiltinFunction::Assert, "assert", DataType::Bool),
        copy_func: make_a_a_func(program, BuiltinFunction::Copy, "!copy"),
        return_func: make_blank_func(program, BuiltinFunction::Return, "!return"),

        automatic_type: automatic_type,
        bool_type: bool_type,
        int_type: int_type,
        float_type: float_type,
        void_type: void_type,
    };

    builtins
}