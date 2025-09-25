use super::{Diff, DiffElement, Path};
use crate::Node;

/// Produces a replacement diff element for non-container nodes.
#[allow(clippy::needless_pass_by_value)]
pub(super) fn diff_primitives(lhs: &Node, rhs: &Node, path: &Path) -> Diff {
    let mut element = DiffElement::new().with_path(path.clone());
    if !matches!(lhs, Node::Void) {
        element.remove.push(lhs.clone());
    }
    if !matches!(rhs, Node::Void) {
        element.add.push(rhs.clone());
    }
    Diff::from_elements(vec![element])
}
