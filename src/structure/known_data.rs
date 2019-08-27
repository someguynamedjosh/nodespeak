#[derive(Clone, Debug)]
pub enum KnownData {
    Unknown,
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<KnownData>),
}
