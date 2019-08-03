use crate::vague::{Entity, EntityId, Program};

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
}

// Adds built-in methods to the root scope.
pub fn add_builtins(program: &mut Program) -> Builtins {
    let scope = program.get_root_scope();
    let builtins = Builtins {
        add_func: program.adopt_entity(Entity::BuiltinFunction("add".to_owned())),
        sub_func: program.adopt_entity(Entity::BuiltinFunction("sub".to_owned())),
        mul_func: program.adopt_entity(Entity::BuiltinFunction("mul".to_owned())),
        div_func: program.adopt_entity(Entity::BuiltinFunction("div".to_owned())),
        int_div_func: program.adopt_entity(Entity::BuiltinFunction("int_div".to_owned())),
        recip_func: program.adopt_entity(Entity::BuiltinFunction("recip".to_owned())),
        mod_func: program.adopt_entity(Entity::BuiltinFunction("mod".to_owned())),
        pow_func: program.adopt_entity(Entity::BuiltinFunction("pow".to_owned())),

        band_func: program.adopt_entity(Entity::BuiltinFunction("band".to_owned())),
        bor_func: program.adopt_entity(Entity::BuiltinFunction("bor".to_owned())),
        bxor_func: program.adopt_entity(Entity::BuiltinFunction("bxor".to_owned())),
        bnot_func: program.adopt_entity(Entity::BuiltinFunction("bnot".to_owned())),

        int_to_float_func: program.adopt_entity(Entity::BuiltinFunction("int_to_float".to_owned())),
        bool_to_float_func: program
            .adopt_entity(Entity::BuiltinFunction("bool_to_float".to_owned())),
        bool_to_int_func: program.adopt_entity(Entity::BuiltinFunction("bool_to_int".to_owned())),
        int_to_bool_func: program.adopt_entity(Entity::BuiltinFunction("int_to_bool".to_owned())),
        float_to_int_func: program.adopt_entity(Entity::BuiltinFunction("float_to_int".to_owned())),
        float_to_bool_func: program
            .adopt_entity(Entity::BuiltinFunction("float_to_bool".to_owned())),

        eq_func: program.adopt_entity(Entity::BuiltinFunction("eq".to_owned())),
        neq_func: program.adopt_entity(Entity::BuiltinFunction("neq".to_owned())),
        lte_func: program.adopt_entity(Entity::BuiltinFunction("lte".to_owned())),
        gte_func: program.adopt_entity(Entity::BuiltinFunction("gte".to_owned())),
        lt_func: program.adopt_entity(Entity::BuiltinFunction("lt".to_owned())),
        gt_func: program.adopt_entity(Entity::BuiltinFunction("gt".to_owned())),

        and_func: program.adopt_entity(Entity::BuiltinFunction("and".to_owned())),
        or_func: program.adopt_entity(Entity::BuiltinFunction("or".to_owned())),
        xor_func: program.adopt_entity(Entity::BuiltinFunction("xor".to_owned())),
        not_func: program.adopt_entity(Entity::BuiltinFunction("not".to_owned())),

        return_func: program.adopt_entity(Entity::BuiltinFunction("return".to_owned())),
    };

    // TODO: Once we add support for templates, properly create the inputs
    // and outputs for all these functions.

    program.define_symbol(scope, "!ADD", builtins.add_func);
    program.define_symbol(scope, "!SUB", builtins.sub_func);
    program.define_symbol(scope, "!MUL", builtins.mul_func);
    program.define_symbol(scope, "!DIV", builtins.div_func);
    program.define_symbol(scope, "!INT_DIV", builtins.int_div_func);
    program.define_symbol(scope, "!RECIP", builtins.recip_func);
    program.define_symbol(scope, "!MOD", builtins.mod_func);
    program.define_symbol(scope, "!POW", builtins.pow_func);

    program.define_symbol(scope, "!BAND", builtins.band_func);
    program.define_symbol(scope, "!BOR", builtins.bor_func);
    program.define_symbol(scope, "!BXOR", builtins.bxor_func);
    program.define_symbol(scope, "!BNOT", builtins.bnot_func);

    program.define_symbol(scope, "!INT_TO_FLOAT", builtins.int_to_float_func);
    program.define_symbol(scope, "!BOOL_TO_FLOAT", builtins.bool_to_float_func);
    program.define_symbol(scope, "!BOOL_TO_INT", builtins.bool_to_int_func);
    program.define_symbol(scope, "!INT_TO_BOOL", builtins.int_to_bool_func);
    program.define_symbol(scope, "!FLOAT_TO_INT", builtins.float_to_int_func);
    program.define_symbol(scope, "!FLOAT_TO_BOOL", builtins.float_to_bool_func);

    program.define_symbol(scope, "!EQ", builtins.eq_func);
    program.define_symbol(scope, "!NEQ", builtins.neq_func);
    program.define_symbol(scope, "!LTE", builtins.lte_func);
    program.define_symbol(scope, "!GTE", builtins.gte_func);
    program.define_symbol(scope, "!LT", builtins.lt_func);
    program.define_symbol(scope, "!GT", builtins.gt_func);

    program.define_symbol(scope, "!AND", builtins.and_func);
    program.define_symbol(scope, "!OR", builtins.or_func);
    program.define_symbol(scope, "!XOR", builtins.xor_func);
    program.define_symbol(scope, "!NOT", builtins.not_func);

    program.define_symbol(scope, "!RETURN", builtins.return_func);

    builtins
}
