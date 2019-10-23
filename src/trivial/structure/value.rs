use super::VariableId;

pub enum LiteralData {
    Int(i64),
    Float(f64),
    Bool(bool),
}

pub enum Value {
    Literal(LiteralData),
    Variable(VariableId),
    Index(VariableId, Vec<Value>),
}