use crate::problem::{CompileProblem, FilePosition};
use crate::structure::{
    DataType, FuncCall, KnownData, Program, ScopeId, VarAccess, Variable, VariableId,
};

pub fn biggest_common_type(a: &DataType, b: &DataType) -> DataType {
    // BCT rule 0
    if a == &DataType::Automatic {
        b.clone()
    } else if b == &DataType::Automatic {
        a.clone()
    }
    // BCT rule 1
    else if a == b {
        a.clone()
    // BCT rule 2
    } else if a == &DataType::Float && b == &DataType::Int {
        DataType::Float
    } else if b == &DataType::Float && a == &DataType::Int {
        DataType::Float
    // BCT rule 3
    } else if a == &DataType::Float && b == &DataType::Bool {
        DataType::Float
    } else if b == &DataType::Float && a == &DataType::Bool {
        DataType::Float
    // BCT rule 4
    } else if a == &DataType::Int && b == &DataType::Bool {
        DataType::Int
    } else if b == &DataType::Int && a == &DataType::Bool {
        DataType::Int
    } else {
        DataType::Void
    }
    // TODO: Implement BCT rules 5 and 6, which require using the currently
    // nonfunctioning interpreter to determine the size of array types.
}

pub fn perform_cast(data: KnownData, from: DataType, to: DataType) -> KnownData {
    // Cast rule 1
    if from == to {
        data
    // Cast rule 2
    } else if from == DataType::Int && to == DataType::Float {
        match data {
            KnownData::Int(value) => KnownData::Float(value as f64),
            _ => panic!("Provided data did not match provided type!"),
        }
    // Cast rule 3
    } else if from == DataType::Float && to == DataType::Int {
        match data {
            KnownData::Float(value) => KnownData::Int(value.floor() as i64),
            _ => panic!("Provided data did not match provided type!"),
        }
    } else {
        KnownData::Unknown
    }
    // TODO: Implement rules 4-6, same reason as biggest_common_type.
}

fn simple_cast(
    program: &mut Program,
    scope: ScopeId,
    from: VarAccess,
    to: VarAccess,
    func: VariableId,
) -> Result<(), CompileProblem> {
    // TODO: Real position.
    let mut func_call = FuncCall::new(func, FilePosition::placeholder());
    func_call.add_input(from);
    func_call.add_output(to);
    program.add_func_call(scope, func_call)
}

pub fn create_cast(
    program: &mut Program,
    scope: ScopeId,
    from: VarAccess,
    to: VarAccess,
) -> Result<(), CompileProblem> {
    // Currently we don't have code to manage arrays.
    assert!(from.borrow_indexes().len() == 0);

    let from_type = from.borrow_data_type(program);
    let to_type = to.borrow_data_type(program);
    // Cast rule 1
    if from_type == to_type {
        panic!("Should avoid creating redundant copies.");
    // Cast rule 2
    } else if from_type == &DataType::Int && to_type == &DataType::Float {
        simple_cast(
            program,
            scope,
            from,
            to,
            program.get_builtins().int_to_float_func,
        )
    // Cast rule 3
    } else if from_type == &DataType::Float && to_type == &DataType::Int {
        simple_cast(
            program,
            scope,
            from,
            to,
            program.get_builtins().float_to_int_func,
        )
    } else {
        panic!("TODO real error");
    }
}
