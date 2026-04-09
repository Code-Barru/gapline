//! `.hw` batch file runner — parser and sequential executor.

mod error;
mod executor;
mod parser;

pub use error::RunError;
pub use executor::execute;
pub use parser::parse_hw_file;
