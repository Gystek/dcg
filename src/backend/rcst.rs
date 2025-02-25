//! Rose Concrete Syntax Tree --- intermediary representation for `tree_sitter`'s `Tree`s.
//!
//! `tree_sitter`'s `Tree`s can be deconstructed into (and reconstructed from) `RCSTree`s.

use crate::backend::{
    bcst::BCSTree,
    types::{Data, Metadata},
};
use tree_sitter::Node;

#[derive(Clone, Debug)]
pub(crate) enum List<A: Clone> {
    Cons(A, Box<List<A>>),
    Nil,
}

impl<A: Clone> List<A> {
    pub(crate) fn car(&self) -> Option<A> {
        match self {
            Self::Cons(x, _) => Some(x.clone()),
            Self::Nil => None,
        }
    }

    pub(crate) fn cdr(self) -> List<A> {
        match self {
            Self::Cons(_, xs) => *xs,
            Self::Nil => Self::Nil,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum RCSTree<A: Data, B: Metadata> {
    Leaf(A),
    Node(B, List<Box<RCSTree<A, B>>>),
}

fn bcst_to_cons<A: Data, B: Metadata>(t: BCSTree<A, B>) -> List<Box<RCSTree<A, B>>> {
    match t {
        BCSTree::Leaf(x) => {
            if x == A::nil() {
                List::Nil
            } else {
                panic!("badly formed binary tree: leaf with other value than nilT");
            }
        }
        BCSTree::Node(m, x, y) => {
            if m == B::cons() {
                let rx = (*x).into();
                let rxs = bcst_to_cons(*y);

                List::Cons(Box::new(rx), Box::new(rxs))
            } else {
                panic!("badly formed binary tree: node with other metadata than consT");
            }
        }
    }
}

impl<A: Data, B: Metadata> From<BCSTree<A, B>> for RCSTree<A, B> {
    fn from(t: BCSTree<A, B>) -> Self {
        match t {
            BCSTree::Leaf(x) => RCSTree::Leaf(x),
            BCSTree::Node(m, x, y) => {
                if *y == BCSTree::Leaf(A::nil()) {
                    RCSTree::Node(m, List::Cons(Box::new((*x).into()), Box::new(List::Nil)))
                } else {
                    let rx = (*x).into();
                    let rxs = bcst_to_cons(*y);

                    RCSTree::Node(m, List::Cons(Box::new(rx), Box::new(rxs)))
                }
            }
        }
    }
}

impl<A: Data, B: Metadata> From<Node<'_>> for RCSTree<A, B> {
    fn from(node: Node<'_>) -> RCSTree<A, B> {
        let meta = B::extract(node);

        if node.child_count() == 0 {
            RCSTree::Leaf(A::extract(node))
        } else {
            let mut children = List::Nil;

            for i in 0..node.child_count() {
                let child = node.child(i).unwrap();
                children = List::Cons(Box::new(child.into()), Box::new(children));
            }

            RCSTree::Node(meta, children)
        }
    }
}

fn rcst_to_node<'a, A: Data, B: Metadata>(rcst: RCSTree<A, B>) -> Node<'a> {
    unimplemented!()
}
