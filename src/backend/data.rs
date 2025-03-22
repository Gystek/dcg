//! Representation of `Node` leaf data
use std::ops::Range;

use tree_sitter::Node;

#[derive(Hash, Debug, Clone, PartialEq, Eq)]
pub(crate) struct Data<'a> {
    pub(crate) node_type: Option<u16>,
    pub(crate) range: Range<(usize, usize)>,
    pub(crate) text: &'a str,
    pub(crate) named: bool,
}

impl<'a> Data<'a> {
    pub(crate) fn from(node: Node<'a>, src: &'a str) -> Self {
	let start = node.start_position();
	let end = node.end_position();
	
        Data {
	    node_type: Some(node.kind_id()),
	    range: (start.row, start.column)..(end.row, end.column),
	    text: &src[node.start_byte()..node.end_byte()],
	    named: node.is_named(),
	}
    }
}

pub(crate) const DATA_NIL: Data = Data {
    node_type: None,
    range: (0, 0)..(0, 0),
    text: "",
    named: false,
};
