mod labels;
mod native_data;
mod native_var;

pub use labels::*;
pub use native_data::*;
pub use native_var::*;

// TODO: Move to a more specific file.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProxyMode {
    /// Use this as a regular array index.
    Keep,
    /// Throw the index away.
    Discard,
    /// Replace with index 0/
    Collapse,
}

pub fn apply_proxy_to_index(proxy: &[(usize, ProxyMode)], index: &[usize]) -> Vec<usize> {
    let mut current_dimension = 0;
    let mut result = Vec::new();
    for proxy_dimension in proxy {
        match proxy_dimension.1 {
            ProxyMode::Keep => result.push(index[current_dimension]),
            ProxyMode::Discard => (),
            ProxyMode::Collapse => result.push(0),
        }
        current_dimension += 1;
    }
    result
}
