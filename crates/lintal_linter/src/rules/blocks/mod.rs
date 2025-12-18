//! Blocks rules for checking brace placement and block structure.

pub mod common;
pub mod left_curly;
pub mod right_curly;

pub use left_curly::LeftCurly;
pub use right_curly::RightCurly;

// Additional rules will be added as they're implemented
