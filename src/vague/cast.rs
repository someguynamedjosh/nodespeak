use crate::vague::{DataType, KnownData};

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
    } else if from == DataType::Int && to == DataType::Float {
        match data {
            KnownData::Int(value) => KnownData::Float(value as f64),
            _ => panic!("Provided data did not match provided type!"),
        }
    // Cast rule 2
    } else if from == DataType::Float && to == DataType::Int {
        match data {
            KnownData::Float(value) => KnownData::Int(value.floor() as i64),
            _ => panic!("Provided data did not match provided type!"),
        }
    } else {
        KnownData::Empty
    }
    // TODO: Implement rules 4-6, same reason as biggest_common_type.
}
