mod ingest;
#[cfg(not(feature = "no-llvmir"))]
mod make_llvm;
pub mod structure;

pub use ingest::ingest;
#[cfg(not(feature = "no-llvmir"))]
pub use make_llvm::*;
