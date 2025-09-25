use std::collections::BTreeMap;

use super::{diff_impl, Diff, DiffElement, Path, PathSegment};
use crate::{DiffOptions, Node};

pub(super) fn diff_objects(
    lhs: &BTreeMap<String, Node>,
    rhs: &BTreeMap<String, Node>,
    path: &Path,
    options: &DiffOptions,
) -> Diff {
    let mut elements = Vec::new();

    let mut lhs_keys: Vec<_> = lhs.keys().cloned().collect();
    lhs_keys.sort();
    for key in lhs_keys {
        let value = &lhs[&key];
        if let Some(other) = rhs.get(&key) {
            let sub_path = path.clone().with_segment(PathSegment::key(key));
            let diff = diff_impl(value, other, &sub_path, options);
            elements.extend(diff.into_iter());
        } else {
            let element = DiffElement::new()
                .with_path(path.clone().with_segment(PathSegment::key(key)))
                .with_remove(vec![value.clone()]);
            elements.push(element);
        }
    }

    let mut rhs_keys: Vec<_> = rhs.keys().cloned().collect();
    rhs_keys.sort();
    for key in rhs_keys {
        if lhs.contains_key(&key) {
            continue;
        }
        let element = DiffElement::new()
            .with_path(path.clone().with_segment(PathSegment::key(key.clone())))
            .with_add(vec![rhs[&key].clone()]);
        elements.push(element);
    }

    Diff::from_elements(elements)
}
