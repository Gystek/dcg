//! Rose Concrete Syntax Tree - IR for tree_sitter `Node`s
use crate::backend::{
    bcst::{BCSTree, LEAF_NIL},
    data::{Data, DATA_NIL},
    metadata::{Metadata, META_CONS},
};
use std::rc::Rc;
use tree_sitter::Node;

#[derive(Clone, Debug)]
pub(crate) enum List<A> {
    Cons(A, Rc<List<A>>),
    Nil,
}

impl<A> List<A> {
    pub(crate) fn car(&self) -> Option<&A> {
        match self {
            Self::Nil => None,
            Self::Cons(x, _) => Some(x),
        }
    }

    pub(crate) fn cdr(&self) -> Option<Rc<List<A>>> {
        match self {
            Self::Nil => None,
            Self::Cons(_, xs) => Some(xs.clone()),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum RCSTree<'a> {
    Leaf(Data<'a>),
    Node(Metadata, List<Rc<RCSTree<'a>>>),
}

impl<'a> RCSTree<'a> {
    pub(crate) fn from(node: Node<'a>, source: &'a str) -> Self {
        if node.child_count() == 0 {
            RCSTree::Leaf(Data::from(node, source))
        } else {
            let mut children = List::Nil;

            for i in (0..node.child_count()).rev() {
                let child = node.child(i).unwrap();

                children = List::Cons(Rc::new(RCSTree::from(child, source)), Rc::new(children));
            }

            RCSTree::Node(Metadata::from(node), children)
        }
    }
}
