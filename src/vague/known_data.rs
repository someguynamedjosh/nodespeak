#[derive(Debug)]
pub enum KnownData {
    Empty,
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<KnownData>)
}