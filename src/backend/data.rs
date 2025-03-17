use tree_sitter::Node;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Data;

impl<'a> From<Node<'a>> for Data {
    fn from(node: Node<'a>) -> Self {
        Data
    }
}

impl Data {
    pub(crate) fn to_node<'a>(self) -> Node<'a> {
        todo!()
    }
}

pub(crate) const DATA_NIL: Data = Data;
