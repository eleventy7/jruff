//! UnusedImports rule implementation.
//!
//! Detects imports that are never used in the code.
//!
//! Checkstyle equivalent: UnusedImportsCheck

use std::collections::HashSet;

use lintal_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use lintal_java_cst::CstNode;
use lintal_source_file::LineIndex;
use lintal_text_size::{TextRange, TextSize};

use crate::{CheckContext, FromConfig, Properties, Rule};

use super::common::{collect_imports, ImportInfo};

/// Violation: import is unused.
#[derive(Debug, Clone)]
pub struct UnusedImportViolation {
    pub import_path: String,
}

impl Violation for UnusedImportViolation {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Always;

    fn message(&self) -> String {
        format!("Unused import - {}.", self.import_path)
    }
}

/// Configuration for UnusedImports rule.
#[derive(Debug, Clone)]
pub struct UnusedImports {
    /// Whether to scan Javadoc comments for type references.
    pub process_javadoc: bool,
}

impl Default for UnusedImports {
    fn default() -> Self {
        Self {
            process_javadoc: true,
        }
    }
}

impl FromConfig for UnusedImports {
    const MODULE_NAME: &'static str = "UnusedImports";

    fn from_config(properties: &Properties) -> Self {
        let process_javadoc = properties
            .get("processJavadoc")
            .map(|v| *v != "false")
            .unwrap_or(true);

        Self { process_javadoc }
    }
}

impl Rule for UnusedImports {
    fn name(&self) -> &'static str {
        "UnusedImports"
    }

    fn check(&self, ctx: &CheckContext, node: &CstNode) -> Vec<Diagnostic> {
        // Only check at program level (once per file)
        if node.kind() != "program" {
            return vec![];
        }

        // TODO: Implement in next task
        vec![]
    }
}
