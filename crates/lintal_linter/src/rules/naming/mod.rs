//! Naming convention rules.
//!
//! These rules check that various Java identifiers follow naming conventions.

// Allow nested if statements - this pattern is readable for checking
// node kind before extracting optional fields
#![allow(clippy::collapsible_if)]

mod constant_name;
mod local_final_variable_name;
mod local_variable_name;
mod member_name;
mod method_name;
mod package_name;
mod parameter_name;
mod static_variable_name;
mod type_name;

pub use constant_name::ConstantName;
pub use local_final_variable_name::LocalFinalVariableName;
pub use local_variable_name::LocalVariableName;
pub use member_name::MemberName;
pub use method_name::MethodName;
pub use package_name::PackageName;
pub use parameter_name::ParameterName;
pub use static_variable_name::StaticVariableName;
pub use type_name::TypeName;
