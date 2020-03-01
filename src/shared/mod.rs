mod native_var;

pub use native_var::*;

// TODO: Move to a more specific file.
#[derive(Clone, Copy, PartialEq)]
pub enum ProxyMode {
    /// Use this as a regular array index.
    Keep,
    /// Throw the index away.
    Discard,
    /// Replace with index 0/
    Collapse,
}
