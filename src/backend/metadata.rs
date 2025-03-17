use tree_sitter::Node;

#[derive(Hash, Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct Metadata;

impl<'a> From<Node<'a>> for Metadata {
    fn from(node: Node<'a>) -> Self {
        Metadata
    }
}

impl Metadata {
    pub(crate) fn apply<'a>(self, node: Node<'a>) -> Node<'a> {
        node
    }
}

pub(crate) const META_CONS: Metadata = Metadata;
