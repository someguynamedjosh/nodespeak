use crate::trivial::{LabelId, VariableId};

pub enum Condition {
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
    Unconditional,
}

pub enum Instruction {
    Move {
        from: VariableId,
        to: VariableId,
    },

    AddI {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },
    SubI {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },
    MulI {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },
    DivI {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },

    AddF {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },
    SubF {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },
    MulF {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },
    DivF {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },

    And {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },
    Or {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },
    Xor {
        a: VariableId,
        b: VariableId,
        x: VariableId,
    },
    Not {
        a: VariableId,
        x: VariableId,
    },

    Compare {
        a: VariableId,
        b: VariableId,
    },
    Label(LabelId),
    Jump {
        label: LabelId,
        condition: Condition,
    },
}
