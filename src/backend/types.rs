//! Datatypes for the diff/patch/merge algorithms.
use std::fmt::Debug;
use tree_sitter::Node;

/// Metadata from the `tree_sitter` `Tree`s, indicating node types
/// and possibly other information.
///
/// The `cons` method generates a "cons" node type, used for destructuring
/// lists into a binary tree structure.
pub(crate) trait Metadata: Clone + Debug + Eq {
    fn extract(node: Node<'_>) -> Self;

    fn construct<'a>(self) -> Node<'a>;

    fn cons() -> Self;
}

/// Data from the `tree_sitter` `Tree`s.
pub(crate) trait Data: Clone + Debug + Eq {
    fn extract(node: Node<'_>) -> Self;

    fn construct<'a>(self) -> Node<'a>;

    fn nil() -> Self;
}
