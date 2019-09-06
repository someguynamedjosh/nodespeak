use crate::problem::{self, CompileProblem, FilePosition};
use crate::structure::{
    DataType, FuncCall, KnownData, Program, ScopeId, VarAccess, Variable, VariableId,
};

pub fn biggest_common_type(program: &Program, a: &DataType, b: &DataType) -> DataType {
    // BCT rule 1
    if a == &DataType::Automatic {
        b.clone()
    } else if b == &DataType::Automatic {
        a.clone()
    // BCT rule 2
    } else if a.equivalent(b, program) {
        a.clone()
    // BCT rules 3 through 5
    } else if let (
        DataType::Array {
            sizes: sizes1,
            base_type: base_type1,
        },
        DataType::Array {
            sizes: sizes2,
            base_type: base_type2,
        },
    ) = (a, b)
    {
        let mut real_sizes1 = Vec::new();
        let mut real_sizes2 = Vec::new();
        // If any of the size specifications are not specific integers, we can't determine which one
        // is the biggest.
        for size in sizes1 {
            match program.borrow_value_of(size) {
                KnownData::Int(value) => real_sizes1.push(*value),
                _ => return DataType::Void,
            }
        }
        for size in sizes2 {
            match program.borrow_value_of(size) {
                KnownData::Int(value) => real_sizes2.push(*value),
                _ => return DataType::Void,
            }
        }
        // If the second type has higher dimensionality than the first, add extra dimensions (with
        // only 1 element) to the type of the first, according to BCT rule 5.
        if real_sizes1.len() < real_sizes2.len() {
            let gap = real_sizes2.len() - real_sizes1.len();
            for _ in 0..gap {
                real_sizes1.push(1);
            }
        }
        // Same thing the other way around.
        if real_sizes2.len() < real_sizes1.len() {
            let gap = real_sizes1.len() - real_sizes2.len();
            for _ in 0..gap {
                real_sizes2.push(1);
            }
        }
        // Figure out if the two sizes are compatible. While doing so, build a list of the biggest
        // sizes.
        let mut final_sizes = Vec::new();
        for (index, (real_size1, real_size2)) in real_sizes1
            .into_iter()
            .zip(real_sizes2.into_iter())
            .enumerate()
        {
            // BCT rule 3
            if real_size1 == real_size2 {
                final_sizes.push(sizes1[index].clone());
            // BCT rule 4
            } else if real_size1 == 1 {
                final_sizes.push(sizes2[index].clone());
            } else if real_size2 == 1 {
                final_sizes.push(sizes1[index].clone());
            } else {
                return DataType::Void;
            }
        }

        DataType::Array {
            sizes: final_sizes,
            base_type: Box::new(biggest_common_type(program, base_type1, base_type2)),
        }
    // BCT rule 5 special cases
    } else if let DataType::Array { sizes, base_type } = a {
        DataType::Array {
            sizes: sizes.clone(),
            base_type: Box::new(biggest_common_type(program, base_type, b)),
        }
    } else if let DataType::Array { sizes, base_type } = b {
        DataType::Array {
            sizes: sizes.clone(),
            base_type: Box::new(biggest_common_type(program, base_type, a)),
        }
    } else {
        return DataType::Void;
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
    // TODO: Arrays
    assert!(from.borrow_indexes().len() == 0);

    let from_type = from.borrow_data_type(program);
    let to_type = to.borrow_data_type(program);
    // Cast rule 1
    if from_type.equivalent(to_type, program) {
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
        Result::Err(problem::illegal_inflation(
            from.get_position().clone(),
            to.get_position().clone(),
        ))
    }
}
