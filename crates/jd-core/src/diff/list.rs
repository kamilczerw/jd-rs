use super::{diff_impl, Diff, DiffElement, Path, PathSegment};
use crate::hash::HashCode;
use crate::{DiffOptions, Node};

pub(super) fn diff_lists(lhs: &[Node], rhs: &[Node], path: &Path, options: &DiffOptions) -> Diff {
    let lhs_hashes: Vec<HashCode> = lhs.iter().map(|node| node.hash_code(options)).collect();
    let rhs_hashes: Vec<HashCode> = rhs.iter().map(|node| node.hash_code(options)).collect();
    let common = longest_common_subsequence(&lhs_hashes, &rhs_hashes);
    let path_with_placeholder = path.clone().with_segment(PathSegment::index(0));
    let elements = diff_rest(
        lhs,
        rhs,
        0,
        path_with_placeholder,
        &lhs_hashes,
        &rhs_hashes,
        &common,
        &Node::Void,
        options,
    );
    Diff::from_elements(elements)
}

#[allow(clippy::too_many_arguments)]
fn diff_rest(
    lhs: &[Node],
    rhs: &[Node],
    path_index: i64,
    path: Path,
    lhs_hashes: &[HashCode],
    rhs_hashes: &[HashCode],
    common: &[HashCode],
    previous: &Node,
    options: &DiffOptions,
) -> Vec<DiffElement> {
    let mut a_cursor = 0usize;
    let mut b_cursor = 0usize;
    let mut common_cursor = 0usize;
    let mut path_cursor = path_index;
    let path_len = path.len();

    let mut diff = vec![DiffElement::new()
        .with_path(path_now(&path, path_cursor))
        .with_before(vec![previous.clone()])];

    loop {
        match () {
            _ if a_cursor == lhs.len() => {
                while b_cursor < rhs.len() {
                    diff[0].add.push(rhs[b_cursor].clone());
                    b_cursor += 1;
                    path_cursor += 2;
                }
                break;
            }
            _ if b_cursor == rhs.len() => {
                while a_cursor < lhs.len() {
                    diff[0].remove.push(lhs[a_cursor].clone());
                    a_cursor += 1;
                }
                break;
            }
            _ if at_common(lhs_hashes, a_cursor, common)
                && at_common(rhs_hashes, b_cursor, common) =>
            {
                a_cursor += 1;
                b_cursor += 1;
                common_cursor += 1;
                path_cursor += 1;
                break;
            }
            _ if at_common(lhs_hashes, a_cursor, common) => {
                while !at_common(rhs_hashes, b_cursor, common) {
                    diff[0].add.push(rhs[b_cursor].clone());
                    b_cursor += 1;
                    path_cursor += 1;
                }
            }
            _ if at_common(rhs_hashes, b_cursor, common) => {
                while !at_common(lhs_hashes, a_cursor, common) {
                    diff[0].remove.push(lhs[a_cursor].clone());
                    a_cursor += 1;
                }
            }
            _ if same_container_type(&lhs[a_cursor], &rhs[b_cursor]) => {
                let sub_path = path_now(&path, path_cursor);
                let mut sub_diff =
                    diff_impl(&lhs[a_cursor], &rhs[b_cursor], &sub_path, options).into_elements();
                if has_changes(&diff) {
                    diff[0].after = after_context(lhs, a_cursor, common_cursor);
                    diff.append(&mut sub_diff);
                } else {
                    diff = sub_diff;
                }
                a_cursor += 1;
                b_cursor += 1;
                path_cursor += 1;
                break;
            }
            _ => {
                diff[0].remove.push(lhs[a_cursor].clone());
                diff[0].add.push(rhs[b_cursor].clone());
                a_cursor += 1;
                b_cursor += 1;
                path_cursor += 1;
            }
        }
    }

    if !has_changes(&diff) {
        diff.clear();
    } else {
        let single = diff.len() < 2;
        if let Some(first) = diff.first_mut() {
            if first.path.len() <= path_len && single {
                first.after = after_context(lhs, a_cursor, common_cursor);
            }
        }
    }

    if a_cursor == lhs.len() && b_cursor == rhs.len() {
        return diff;
    }

    let previous_node = if b_cursor == 0 { Node::Void } else { rhs[b_cursor - 1].clone() };
    let mut rest = diff_rest(
        &lhs[a_cursor..],
        &rhs[b_cursor..],
        path_cursor,
        path_now(&path, path_cursor),
        &lhs_hashes[a_cursor..],
        &rhs_hashes[b_cursor..],
        &common[common_cursor..],
        &previous_node,
        options,
    );
    diff.append(&mut rest);
    diff
}

fn at_common(hashes: &[HashCode], cursor: usize, common: &[HashCode]) -> bool {
    if cursor >= hashes.len() || common.is_empty() {
        return false;
    }
    hashes[cursor] == common[0]
}

fn has_changes(diff: &[DiffElement]) -> bool {
    diff.first()
        .map(|element| !element.add.is_empty() || !element.remove.is_empty())
        .unwrap_or(false)
}

fn after_context(lhs: &[Node], a_cursor: usize, common_cursor: usize) -> Vec<Node> {
    let index = a_cursor.saturating_sub(common_cursor);
    if index >= lhs.len() {
        vec![Node::Void]
    } else {
        vec![lhs[index].clone()]
    }
}

fn path_now(path: &Path, path_cursor: i64) -> Path {
    path.drop_last().with_segment(PathSegment::index(path_cursor))
}

fn same_container_type(lhs: &Node, rhs: &Node) -> bool {
    matches!(lhs, Node::Object(_)) && matches!(rhs, Node::Object(_))
        || matches!(lhs, Node::Array(_)) && matches!(rhs, Node::Array(_))
}

fn longest_common_subsequence(lhs: &[HashCode], rhs: &[HashCode]) -> Vec<HashCode> {
    let n = lhs.len();
    let m = rhs.len();
    let mut table = vec![vec![0usize; m + 1]; n + 1];
    for (i, lhs_hash) in lhs.iter().enumerate() {
        for (j, rhs_hash) in rhs.iter().enumerate() {
            if lhs_hash == rhs_hash {
                table[i + 1][j + 1] = table[i][j] + 1;
            } else {
                table[i + 1][j + 1] = table[i][j + 1].max(table[i + 1][j]);
            }
        }
    }

    let mut result = Vec::with_capacity(table[n][m]);
    let mut i = n;
    let mut j = m;
    while i > 0 && j > 0 {
        if lhs[i - 1] == rhs[j - 1] {
            result.push(lhs[i - 1]);
            i -= 1;
            j -= 1;
        } else if table[i - 1][j] >= table[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    result.reverse();
    result
}
