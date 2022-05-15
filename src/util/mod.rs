mod iters;
mod nvec;

use std::{rc::Rc, cell::RefCell};

pub use iters::*;
pub use nvec::*;

pub type Rcrc<T> = Rc<RefCell<T>>;

pub fn rcrc<T>(value: T) -> Rcrc<T> {
    Rc::new(RefCell::new(value))
}
