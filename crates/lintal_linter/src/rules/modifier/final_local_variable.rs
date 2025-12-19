//! FinalLocalVariable rule - checks that local variables that are never reassigned should be final.
//!
//! This is a complex stateful rule that tracks variable declarations and assignments.

use crate::{CheckContext, FromConfig, Rule};
use lintal_diagnostics::{Diagnostic, FixAvailability, Violation};
use lintal_java_cst::CstNode;
use lintal_text_size::TextRange;
use std::collections::HashMap;

/// Checks that local variables that are never reassigned are declared final.
pub struct FinalLocalVariable {
    #[allow(dead_code)] // Will be used in later tasks for enhanced for loop support
    validate_enhanced_for_loop_variable: bool,
    validate_unnamed_variables: bool,
}

/// Violation for a variable that should be final.
#[derive(Debug, Clone)]
pub struct VariableShouldBeFinal {
    pub var_name: String,
}

impl Violation for VariableShouldBeFinal {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    fn message(&self) -> String {
        format!("Variable '{}' should be declared final.", self.var_name)
    }
}

impl FromConfig for FinalLocalVariable {
    const MODULE_NAME: &'static str = "FinalLocalVariable";

    fn from_config(properties: &HashMap<&str, &str>) -> Self {
        let validate_enhanced_for_loop_variable = properties
            .get("validateEnhancedForLoopVariable")
            .map(|v| *v == "true")
            .unwrap_or(false);

        let validate_unnamed_variables = properties
            .get("validateUnnamedVariables")
            .map(|v| *v == "true")
            .unwrap_or(false);

        Self {
            validate_enhanced_for_loop_variable,
            validate_unnamed_variables,
        }
    }
}

/// Candidate variable that might need to be final.
#[derive(Debug, Clone)]
struct VariableCandidate {
    /// The range of the identifier in the source
    ident_range: TextRange,
    /// The name of the variable
    name: String,
    /// Whether this variable has been assigned (not including initialization)
    assigned: bool,
    /// Whether this variable has been assigned more than once
    already_assigned: bool,
}

/// Data for a single scope (method, constructor, block, etc.)
#[derive(Debug)]
struct ScopeData {
    /// Map of variable name to candidate
    variables: HashMap<String, VariableCandidate>,
}

impl ScopeData {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Add a variable declaration to this scope.
    fn add_variable(&mut self, name: String, ident_range: TextRange) {
        self.variables.insert(
            name.clone(),
            VariableCandidate {
                ident_range,
                name,
                assigned: false,
                already_assigned: false,
            },
        );
    }

    /// Mark a variable as assigned.
    /// If it was already assigned, mark it as already_assigned (not a candidate for final).
    fn mark_assigned(&mut self, name: &str) {
        if let Some(candidate) = self.variables.get_mut(name) {
            if candidate.assigned {
                candidate.already_assigned = true;
            } else {
                candidate.assigned = true;
            }
        }
    }

    /// Get all variables that should be final (never assigned after initialization).
    fn get_should_be_final(&self) -> Vec<&VariableCandidate> {
        self.variables
            .values()
            .filter(|v| !v.assigned && !v.already_assigned)
            .collect()
    }
}

/// Visitor that processes a method/constructor/block body.
struct FinalLocalVariableVisitor<'a> {
    rule: &'a FinalLocalVariable,
    ctx: &'a CheckContext<'a>,
    /// Stack of scopes
    scopes: Vec<ScopeData>,
    /// Diagnostics collected
    diagnostics: Vec<Diagnostic>,
}

impl<'a> FinalLocalVariableVisitor<'a> {
    fn new(rule: &'a FinalLocalVariable, ctx: &'a CheckContext<'a>) -> Self {
        Self {
            rule,
            ctx,
            scopes: vec![],
            diagnostics: vec![],
        }
    }

    /// Push a new scope.
    fn push_scope(&mut self) {
        self.scopes.push(ScopeData::new());
    }

    /// Pop a scope and report violations for variables that should be final.
    fn pop_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            for candidate in scope.get_should_be_final() {
                self.report_violation(candidate.ident_range, &candidate.name);
            }
        }
    }

    /// Get the current scope.
    fn current_scope(&mut self) -> Option<&mut ScopeData> {
        self.scopes.last_mut()
    }

    /// Report a violation for a variable that should be final.
    fn report_violation(&mut self, ident_range: TextRange, var_name: &str) {
        let diagnostic = Diagnostic::new(
            VariableShouldBeFinal {
                var_name: var_name.to_string(),
            },
            ident_range,
        );
        self.diagnostics.push(diagnostic);
    }

    /// Visit a node and process it.
    fn visit(&mut self, node: &CstNode) {
        match node.kind() {
            "local_variable_declaration" => {
                self.process_variable_declaration(node);
                self.visit_children(node);
            }
            "assignment_expression" => {
                self.process_assignment(node);
                self.visit_children(node);
            }
            "update_expression" => {
                self.process_update_expression(node);
                self.visit_children(node);
            }
            _ => {
                self.visit_children(node);
            }
        }
    }

    /// Visit all children of a node.
    fn visit_children(&mut self, node: &CstNode) {
        for child in node.children() {
            self.visit(&child);
        }
    }

    /// Process a variable declaration.
    fn process_variable_declaration(&mut self, node: &CstNode) {
        // Check if already has final modifier
        // Note: modifiers might not be a field, check children
        for child in node.children() {
            if child.kind() == "modifiers" {
                if super::common::has_modifier(&child, "final") {
                    return; // Already final, skip
                }
            } else if child.kind() == "final" {
                // Sometimes final appears directly as a child
                return;
            }
        }

        // Find all variable declarators
        for child in node.children() {
            if child.kind() == "variable_declarator"
                && let Some(name_node) = child.child_by_field_name("name")
            {
                let var_name = &self.ctx.source()[name_node.range()];

                // Skip unnamed variables if configured
                if !self.rule.validate_unnamed_variables && var_name == "_" {
                    continue;
                }

                // Add to current scope
                if let Some(scope) = self.current_scope() {
                    scope.add_variable(var_name.to_string(), name_node.range());
                }
            }
        }
    }

    /// Process an assignment expression.
    fn process_assignment(&mut self, node: &CstNode) {
        if let Some(left) = node.child_by_field_name("left")
            && left.kind() == "identifier"
        {
            let var_name = &self.ctx.source()[left.range()];
            // Mark as assigned in all scopes (check from innermost to outermost)
            for scope in self.scopes.iter_mut().rev() {
                if scope.variables.contains_key(var_name) {
                    scope.mark_assigned(var_name);
                    break;
                }
            }
        }
    }

    /// Process an update expression (++, --).
    fn process_update_expression(&mut self, node: &CstNode) {
        // The update_expression has the form: expression ++ or ++ expression
        // We need to find the identifier being updated
        if let Some(expr) = node.child_by_field_name("argument") {
            if expr.kind() == "identifier" {
                let var_name = &self.ctx.source()[expr.range()];
                // Mark as assigned in all scopes
                for scope in self.scopes.iter_mut().rev() {
                    if scope.variables.contains_key(var_name) {
                        scope.mark_assigned(var_name);
                        break;
                    }
                }
            }
        }
        // Fallback: check all children
        else {
            for child in node.children() {
                if child.kind() == "identifier" {
                    let var_name = &self.ctx.source()[child.range()];
                    for scope in self.scopes.iter_mut().rev() {
                        if scope.variables.contains_key(var_name) {
                            scope.mark_assigned(var_name);
                            break;
                        }
                    }
                    break;
                }
            }
        }
    }
}

impl Rule for FinalLocalVariable {
    fn name(&self) -> &'static str {
        "FinalLocalVariable"
    }

    fn check(&self, ctx: &CheckContext, node: &CstNode) -> Vec<Diagnostic> {
        // Only process at the top-level nodes that establish scopes
        match node.kind() {
            "method_declaration" | "constructor_declaration" => {
                if let Some(body) = node.child_by_field_name("body") {
                    let mut visitor = FinalLocalVariableVisitor::new(self, ctx);
                    visitor.push_scope();
                    visitor.visit(&body);
                    visitor.pop_scope();
                    return visitor.diagnostics;
                }
            }
            "static_initializer" => {
                // Static initializer block - find the block child
                for child in node.children() {
                    if child.kind() == "block" {
                        let mut visitor = FinalLocalVariableVisitor::new(self, ctx);
                        visitor.push_scope();
                        visitor.visit(&child);
                        visitor.pop_scope();
                        return visitor.diagnostics;
                    }
                }
            }
            "block" => {
                // Only process instance initializer blocks (parent is class_body)
                if let Some(parent) = node.parent()
                    && parent.kind() == "class_body"
                {
                    let mut visitor = FinalLocalVariableVisitor::new(self, ctx);
                    visitor.push_scope();
                    visitor.visit(node);
                    visitor.pop_scope();
                    return visitor.diagnostics;
                }
            }
            _ => {}
        }
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_config_defaults() {
        let properties = HashMap::new();
        let rule = FinalLocalVariable::from_config(&properties);
        assert!(!rule.validate_enhanced_for_loop_variable);
        assert!(!rule.validate_unnamed_variables);
    }

    #[test]
    fn test_from_config_custom() {
        let mut properties = HashMap::new();
        properties.insert("validateEnhancedForLoopVariable", "true");
        properties.insert("validateUnnamedVariables", "true");
        let rule = FinalLocalVariable::from_config(&properties);
        assert!(rule.validate_enhanced_for_loop_variable);
        assert!(rule.validate_unnamed_variables);
    }
}
