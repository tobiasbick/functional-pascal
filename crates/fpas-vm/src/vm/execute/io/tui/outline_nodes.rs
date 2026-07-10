//! Decode FPAS `OutlineNode` records and build turbo-vision outline trees.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use super::try2::handle_records::TUI_OUTLINE_NODE_TYPE;
use crate::vm::Worker;
use crate::vm::diagnostics::{TYPE_MISMATCH_CODE, VmError, runtime_error};
use crate::vm::shared::TurboVisionOutlineNode;
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use std::cell::RefCell;
use std::rc::Rc;
use turbo_vision::views::outline::Node;

impl Worker {
    /// Pop an array of `Std.Tui.OutlineNode` records from the VM stack.
    pub(in crate::vm::execute::io::tui) fn pop_outline_roots(
        &mut self,
        label: &'static str,
        line: SourceLocation,
    ) -> Result<Vec<TurboVisionOutlineNode>, VmError> {
        match self.pop(line)? {
            Value::Array(values) => values
                .into_iter()
                .enumerate()
                .map(|(index, value)| decode_outline_node(value, label, index, line))
                .collect(),
            other => Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!(
                    "{label} must be array of OutlineNode, got {}",
                    other.type_name()
                ),
                "Pass an array of Std.Tui.OutlineNode records.",
                line,
            )),
        }
    }
}

/// Decode one `Std.Tui.OutlineNode` record from a runtime value.
pub(in crate::vm::execute::io::tui) fn decode_outline_node(
    value: Value,
    label: &'static str,
    index: usize,
    line: SourceLocation,
) -> Result<TurboVisionOutlineNode, VmError> {
    let Value::Record { type_name, fields } = value else {
        return Err(runtime_error(
            TYPE_MISMATCH_CODE,
            format!(
                "{label}[{index}] must be OutlineNode, got {}",
                value.type_name()
            ),
            "Pass Std.Tui.OutlineNode records with text, children, and expanded fields.",
            line,
        ));
    };
    if type_name != TUI_OUTLINE_NODE_TYPE {
        return Err(runtime_error(
            TYPE_MISMATCH_CODE,
            format!("{label}[{index}] expected {TUI_OUTLINE_NODE_TYPE}, got {type_name}"),
            "Pass Std.Tui.OutlineNode records.",
            line,
        ));
    }

    let text = string_field(&fields, "text", label, index, line)?;
    let expanded = bool_field(&fields, "expanded", label, index, line)?;
    let children = children_field(&fields, label, index, line)?;

    Ok(TurboVisionOutlineNode {
        text,
        children,
        expanded,
    })
}

fn children_field(
    fields: &[(String, Value)],
    label: &'static str,
    index: usize,
    line: SourceLocation,
) -> Result<Vec<TurboVisionOutlineNode>, VmError> {
    let value = fields
        .iter()
        .find(|(name, _)| name == "children")
        .map(|(_, value)| value)
        .ok_or_else(|| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("{label}[{index}].children is missing"),
                "Pass an OutlineNode record with a children array.",
                line,
            )
        })?;
    let Value::Array(values) = value else {
        return Err(runtime_error(
            TYPE_MISMATCH_CODE,
            format!(
                "{label}[{index}].children must be array, got {}",
                value.type_name()
            ),
            "Pass an array of Std.Tui.OutlineNode records.",
            line,
        ));
    };

    values
        .iter()
        .cloned()
        .enumerate()
        .map(|(child_index, child)| decode_outline_node(child, label, child_index, line))
        .collect()
}

fn string_field(
    fields: &[(String, Value)],
    name: &str,
    label: &'static str,
    index: usize,
    line: SourceLocation,
) -> Result<String, VmError> {
    let value = fields
        .iter()
        .find(|(field, _)| field == name)
        .map(|(_, value)| value)
        .ok_or_else(|| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("{label}[{index}].{name} is missing"),
                "Pass an OutlineNode record with text, children, and expanded fields.",
                line,
            )
        })?;
    match value {
        Value::Str(text) => Ok(text.clone()),
        other => Err(runtime_error(
            TYPE_MISMATCH_CODE,
            format!(
                "{label}[{index}].{name} must be string, got {}",
                other.type_name()
            ),
            "Pass a string label for the outline node.",
            line,
        )),
    }
}

fn bool_field(
    fields: &[(String, Value)],
    name: &str,
    label: &'static str,
    index: usize,
    line: SourceLocation,
) -> Result<bool, VmError> {
    let value = fields
        .iter()
        .find(|(field, _)| field == name)
        .map(|(_, value)| value)
        .ok_or_else(|| {
            runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("{label}[{index}].{name} is missing"),
                "Pass an OutlineNode record with text, children, and expanded fields.",
                line,
            )
        })?;
    match value {
        Value::Boolean(value) => Ok(*value),
        other => Err(runtime_error(
            TYPE_MISMATCH_CODE,
            format!(
                "{label}[{index}].{name} must be boolean, got {}",
                other.type_name()
            ),
            "Pass true to show child nodes, or false to collapse them.",
            line,
        )),
    }
}

/// Build upstream turbo-vision outline roots from FPAS node data.
pub(in crate::vm::execute::io::tui) fn build_outline_tv_roots(
    roots: &[TurboVisionOutlineNode],
) -> Vec<Rc<RefCell<Node<String>>>> {
    roots.iter().map(build_outline_tv_node).collect()
}

fn build_outline_tv_node(node: &TurboVisionOutlineNode) -> Rc<RefCell<Node<String>>> {
    let rc = Rc::new(RefCell::new(Node::new(node.text.clone())));
    {
        let mut borrow = rc.borrow_mut();
        borrow.expanded = node.expanded;
        for child in &node.children {
            borrow.add_child(build_outline_tv_node(child));
        }
    }
    rc
}

/// Initial flat selection index for a newly built outline.
pub(in crate::vm::execute::io::tui) fn initial_outline_selection(
    roots: &[TurboVisionOutlineNode],
) -> Option<usize> {
    if flatten_visible_labels(roots).is_empty() {
        None
    } else {
        Some(0)
    }
}

/// Flatten visible node labels in the same order as `OutlineViewer`.
pub(in crate::vm::execute::io::tui) fn flatten_visible_labels(
    roots: &[TurboVisionOutlineNode],
) -> Vec<String> {
    let mut labels = Vec::new();
    for (index, root) in roots.iter().enumerate() {
        let is_last_root = index + 1 == roots.len();
        flatten_visible_node(root, &mut labels, is_last_root);
    }
    labels
}

fn flatten_visible_node(node: &TurboVisionOutlineNode, labels: &mut Vec<String>, _is_last: bool) {
    labels.push(node.text.clone());
    if node.expanded {
        for (index, child) in node.children.iter().enumerate() {
            let is_last_child = index + 1 == node.children.len();
            flatten_visible_node(child, labels, is_last_child);
        }
    }
}

/// Label at a flat visible index, if any.
pub(in crate::vm::execute::io::tui) fn outline_label_at_flat_index(
    roots: &[TurboVisionOutlineNode],
    index: usize,
) -> Option<String> {
    flatten_visible_labels(roots).into_iter().nth(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten_respects_expanded_state() {
        let roots = vec![TurboVisionOutlineNode {
            text: "root".into(),
            expanded: false,
            children: vec![TurboVisionOutlineNode {
                text: "hidden".into(),
                expanded: false,
                children: Vec::new(),
            }],
        }];

        assert_eq!(flatten_visible_labels(&roots), vec!["root".to_string()]);

        let expanded = vec![TurboVisionOutlineNode {
            text: "root".into(),
            expanded: true,
            children: vec![TurboVisionOutlineNode {
                text: "visible".into(),
                expanded: false,
                children: Vec::new(),
            }],
        }];
        assert_eq!(
            flatten_visible_labels(&expanded),
            vec!["root".to_string(), "visible".to_string()]
        );
    }
}
