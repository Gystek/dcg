//! Binary Concrete Syntax Tree --- binary representation for `tree_sitter`'s `Tree`s
//!
//! `BCSTree` should only be constructed from (and destructed into) `RCSTree`s.

use crate::backend::{
    rcst::{List, RCSTree},
    types::{Data, Metadata},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BCSTree<A: Data, B: Metadata> {
    Leaf(A),
    Node(B, Box<BCSTree<A, B>>, Box<BCSTree<A, B>>),
}

impl<A: Data, B: Metadata> BCSTree<A, B> {
    pub(crate) fn size(&self) -> usize {
	match self {
	    Self::Leaf(_) => 1,
	    Self::Node(_, x, y) => 1 + x.size() + y.size(),
	}
    }
}

impl<A: Data, B: Metadata> From<RCSTree<A, B>> for BCSTree<A, B> {
    fn from(t: RCSTree<A, B>) -> Self {
        match t {
            RCSTree::Leaf(x) => BCSTree::Leaf(x),
            RCSTree::Node(m, children) => {
                if let Some(x) = children.car() {
                    let xs = children.cdr();

                    if let List::Nil = xs {
                        BCSTree::Node(m, Box::new((*x).into()), Box::new(BCSTree::Leaf(A::nil())))
                    } else {
                        let bx = (*x).into();
                        let bxs = RCSTree::Node(B::cons(), xs).into();

                        BCSTree::Node(m, Box::new(bx), Box::new(bxs))
                    }
                } else {
                    BCSTree::Node(
                        m,
                        Box::new(BCSTree::Leaf(A::nil())),
                        Box::new(BCSTree::Leaf(A::nil())),
                    )
                }
            }
        }
    }
}
