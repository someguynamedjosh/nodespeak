use crate::vague::{
    make_var, BuiltinFunction, BuiltinFunctionEntity, DataType, Entity, EntityId, Program,
    VariableEntity,
};

#[readonly::make]
#[derive(Debug)]
pub struct Builtins {
    pub add_func: EntityId,
    pub sub_func: EntityId,
    pub mul_func: EntityId,
    pub div_func: EntityId,
    pub int_div_func: EntityId,
    pub recip_func: EntityId,
    pub mod_func: EntityId,
    pub pow_func: EntityId,

    pub band_func: EntityId,
    pub bor_func: EntityId,
    pub bxor_func: EntityId,
    pub bnot_func: EntityId,

    pub int_to_float_func: EntityId,
    pub bool_to_float_func: EntityId,
    pub bool_to_int_func: EntityId,
    pub int_to_bool_func: EntityId,
    pub float_to_int_func: EntityId,
    pub float_to_bool_func: EntityId,

    pub eq_func: EntityId,
    pub neq_func: EntityId,
    pub lte_func: EntityId,
    pub gte_func: EntityId,
    pub lt_func: EntityId,
    pub gt_func: EntityId,

    pub and_func: EntityId,
    pub or_func: EntityId,
    pub xor_func: EntityId,
    pub not_func: EntityId,

    pub return_func: EntityId,

    pub automatic_type: EntityId,
    pub bool_type: EntityId,
    pub int_type: EntityId,
    pub float_type: EntityId,
}

// Adds built-in methods to the root scope.
pub fn add_builtins(program: &mut Program) -> Builtins {
    let scope = program.get_root_scope();

    let automatic_type =
        program.adopt_and_define_symbol(scope, "Auto", Entity::DataType(DataType::Automatic));
    let bool_type =
        program.adopt_and_define_symbol(scope, "Bool", Entity::DataType(DataType::Bool));
    let int_type = program.adopt_and_define_symbol(scope, "Int", Entity::DataType(DataType::Int));
    let float_type =
        program.adopt_and_define_symbol(scope, "Float", Entity::DataType(DataType::Float));

    let make_aa_a_func = |program: &mut Program, func: BuiltinFunction, name: &str| {
        let mut bfe = BuiltinFunctionEntity::new(func, program);
        let parameter = program.adopt_and_define_symbol(
            bfe.get_scope(),
            "TYPE",
            Entity::DataType(DataType::AwaitingTemplate),
        );
        let in1 = program.adopt_and_define_symbol(bfe.get_scope(), "in1", make_var(parameter));
        let in2 = program.adopt_and_define_symbol(bfe.get_scope(), "in2", make_var(parameter));
        let out = program.adopt_and_define_symbol(bfe.get_scope(), "out", make_var(parameter));
        bfe.add_template_parameter(parameter);
        bfe.add_input(in1);
        bfe.add_input(in2);
        bfe.add_output(out);
        program.adopt_and_define_symbol(scope, name, Entity::BuiltinFunction(bfe))
    };

    let make_logic_func = |program: &mut Program, func: BuiltinFunction, name: &str| {
        let mut bfe = BuiltinFunctionEntity::new(func, program);
        let in1 = program.adopt_and_define_symbol(bfe.get_scope(), "in1", make_var(bool_type));
        let in2 = program.adopt_and_define_symbol(bfe.get_scope(), "in2", make_var(bool_type));
        let out = program.adopt_and_define_symbol(bfe.get_scope(), "out", make_var(bool_type));
        bfe.add_input(in1);
        bfe.add_input(in2);
        bfe.add_output(out);
        program.adopt_and_define_symbol(scope, name, Entity::BuiltinFunction(bfe))
    };

    let make_convert_func =
        |program: &mut Program, func: BuiltinFunction, name: &str, in_type, out_type| {
            let mut bfe = BuiltinFunctionEntity::new(func, program);
            let in1 = program.adopt_and_define_symbol(bfe.get_scope(), "in1", make_var(in_type));
            let out = program.adopt_and_define_symbol(bfe.get_scope(), "out", make_var(out_type));
            bfe.add_input(in1);
            bfe.add_output(out);
            program.adopt_and_define_symbol(scope, name, Entity::BuiltinFunction(bfe))
        };

    let make_comparison_func = |program: &mut Program, func: BuiltinFunction, name: &str| {
        let mut bfe = BuiltinFunctionEntity::new(func, program);
        let parameter = program.adopt_and_define_symbol(
            bfe.get_scope(),
            "TYPE",
            Entity::DataType(DataType::AwaitingTemplate),
        );
        let in1 = program.adopt_and_define_symbol(bfe.get_scope(), "compare", make_var(parameter));
        let in2 = program.adopt_and_define_symbol(bfe.get_scope(), "to", make_var(parameter));
        let out = program.adopt_and_define_symbol(bfe.get_scope(), "result", make_var(bool_type));
        bfe.add_template_parameter(parameter);
        bfe.add_input(in1);
        bfe.add_input(in2);
        bfe.add_output(out);
        program.adopt_and_define_symbol(scope, name, Entity::BuiltinFunction(bfe))
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
            int_type,
            float_type,
        ),
        float_to_int_func: make_convert_func(
            program,
            BuiltinFunction::FloatToInt,
            "!float_to_int",
            float_type,
            int_type,
        ),
        bool_to_float_func: make_convert_func(
            program,
            BuiltinFunction::BoolToFloat,
            "!bool_to_float",
            bool_type,
            float_type,
        ),
        float_to_bool_func: make_convert_func(
            program,
            BuiltinFunction::FloatToBool,
            "!float_to_bool",
            float_type,
            bool_type,
        ),
        bool_to_int_func: make_convert_func(
            program,
            BuiltinFunction::BoolToInt,
            "!bool_to_int",
            bool_type,
            int_type,
        ),
        int_to_bool_func: make_convert_func(
            program,
            BuiltinFunction::IntToBool,
            "!int_to_bool",
            int_type,
            bool_type,
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

        // TODO: A proper return signature for this absolute mess of an all-purpose function.
        return_func: make_aa_a_func(program, BuiltinFunction::Return, "return"),

        automatic_type: automatic_type,
        bool_type: bool_type,
        int_type: int_type,
        float_type: float_type,
    };

    builtins
}
