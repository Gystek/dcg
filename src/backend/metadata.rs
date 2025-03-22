//! Representation of non-terminal `Node` (de facto meta)data.
use tree_sitter::Node;

#[derive(Hash, Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct Metadata {
    pub(crate) node_type: Option<&'static str>,
}

impl<'a> From<Node<'a>> for Metadata {
    fn from(node: Node<'a>) -> Self {
        Metadata {
            node_type: Some(node.kind()),
        }
    }
}

pub(crate) const META_CONS: Metadata = Metadata { node_type: None };
