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

            for i in 0..node.child_count() {
                let child = node.child(i).unwrap();

                children = List::Cons(Rc::new(RCSTree::from(child, source)), Rc::new(children));
            }

            RCSTree::Node(Metadata::from(node), children)
        }
    }
}

fn bin_to_cons<'a>(t: &BCSTree<'a>) -> List<Rc<RCSTree<'a>>> {
    match t {
        BCSTree::Leaf(x) => {
            if x == &DATA_NIL {
                List::Nil
            } else {
                unreachable!("this shouldn't be reachable on well-formed trees");
            }
        }
        BCSTree::Node(m, x, y) => {
            if m == &META_CONS {
                let rx = x.as_ref().clone().into();
                let rxs = bin_to_cons(y);

                List::Cons(Rc::new(rx), Rc::new(rxs))
            } else {
                unreachable!("this shouldn't be reachable on well-formed trees");
            }
        }
    }
}

impl<'a> From<BCSTree<'a>> for RCSTree<'a> {
    fn from(t: BCSTree<'a>) -> Self {
        match t {
            BCSTree::Leaf(x) => RCSTree::Leaf(x),
            BCSTree::Node(m, x, y) => {
                if y.as_ref() == &LEAF_NIL {
                    RCSTree::Node(
                        m,
                        List::Cons(Rc::new(x.as_ref().clone().into()), Rc::new(List::Nil)),
                    )
                } else {
                    let rx = x.as_ref().clone().into();
                    let rxs = bin_to_cons(x.as_ref());

                    RCSTree::Node(m, List::Cons(Rc::new(rx), Rc::new(rxs)))
                }
            }
        }
    }
}
