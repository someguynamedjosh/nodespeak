use crate::problem::{self, CompileProblem, FilePosition};
use crate::structure::{BaseType, DataType, Expression, KnownData, Program, ScopeId, VariableId};

// TODO: A better error instead of just returning void all the time.
pub fn biggest_common_type(program: &Program, a: &DataType, b: &DataType) -> DataType {
    // BCT rule 1
    if a.is_scalar() && a.borrow_base() == &BaseType::Automatic {
        b.clone()
    } else if b.is_scalar() && b.borrow_base() == &BaseType::Automatic {
        a.clone()
    // BCT rule 2
    } else if a.equivalent(b, program) {
        a.clone()
    // BCT rules 3 through 5
    } else if a.is_array() && b.is_array() {
        let mut a_sizes = Vec::new();
        let mut b_sizes = Vec::new();
        // If any of the size specifications are not specific integers, we can't determine which one
        // is the biggest.
        for size in a.borrow_sizes() {
            match program.borrow_value_of(size) {
                KnownData::Int(value) => a_sizes.push(*value),
                _ => return BaseType::Void.to_scalar_type(),
            }
        }
        for size in b.borrow_sizes() {
            match program.borrow_value_of(size) {
                KnownData::Int(value) => b_sizes.push(*value),
                _ => return BaseType::Void.to_scalar_type(),
            }
        }
        // If the second type has higher dimensionality than the first, add extra dimensions (with
        // only 1 element) to the type of the first, according to BCT rule 5.
        if a_sizes.len() < b_sizes.len() {
            let gap = b_sizes.len() - a_sizes.len();
            for _ in 0..gap {
                a_sizes.push(1);
            }
        }
        // Same thing the other way around.
        if b_sizes.len() < a_sizes.len() {
            let gap = a_sizes.len() - b_sizes.len();
            for _ in 0..gap {
                b_sizes.push(1);
            }
        }
        // Figure out if the two sizes are compatible. While doing so, build a list of the biggest
        // sizes.
        let mut final_sizes = Vec::new();
        for (index, (real_size1, real_size2)) in
            a_sizes.into_iter().zip(b_sizes.into_iter()).enumerate()
        {
            // BCT rule 3
            if real_size1 == real_size2 {
                final_sizes.push(a.borrow_sizes()[index].clone());
            // BCT rule 4
            } else if real_size1 == 1 {
                final_sizes.push(b.borrow_sizes()[index].clone());
            } else if real_size2 == 1 {
                final_sizes.push(a.borrow_sizes()[index].clone());
            } else {
                return BaseType::Void.to_scalar_type();
            }
        }

        // For now, there are no base types that exist that can be casted to each other.
        if !a.borrow_base().equivalent(b.borrow_base(), program) {
            return BaseType::Void.to_scalar_type();
        }

        DataType::array(a.borrow_base().clone(), final_sizes)
    // BCT rule 5 special cases
    } else if a.is_array() {
        // For now, there are no base types that exist that can be casted to each other.
        if !a.borrow_base().equivalent(b.borrow_base(), program) {
            return BaseType::Void.to_scalar_type();
        }
        return a.clone();
    } else if b.is_array() {
        // For now, there are no base types that exist that can be casted to each other.
        if !a.borrow_base().equivalent(b.borrow_base(), program) {
            return BaseType::Void.to_scalar_type();
        }
        return b.clone();
    } else {
        return BaseType::Void.to_scalar_type();
    }
    // TODO: Implement BCT rules 5 and 6, which require using the currently
    // nonfunctioning interpreter to determine the size of array types.
}

pub fn create_cast(
    program: &mut Program,
    scope: ScopeId,
    from: Expression,
    to: Expression,
) -> Result<(), CompileProblem> {
    // Currently we don't have code to manage arrays.
    // TODO Arrays
    // TODO Anything.
    Result::Ok(())
}
