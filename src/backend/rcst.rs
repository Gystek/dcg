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
pub(crate) enum RCSTree {
    Leaf(Data),
    Node(Metadata, List<Rc<RCSTree>>),
}

impl<'a> From<Node<'a>> for RCSTree {
    fn from(node: Node<'a>) -> Self {
        let meta = Metadata::from(node);

        if node.child_count() == 0 {
            RCSTree::Leaf(Data::from(node))
        } else {
            let mut children = List::Nil;

            for i in 0..node.child_count() {
                let child = node.child(i).unwrap();

                children = List::Cons(Rc::new(child.into()), Rc::new(children));
            }

            RCSTree::Node(meta, children)
        }
    }
}

fn bin_to_cons(t: &BCSTree) -> List<Rc<RCSTree>> {
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

impl From<BCSTree> for RCSTree {
    fn from(t: BCSTree) -> Self {
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

impl RCSTree {
    pub(crate) fn to_node<'a>(self) -> Node<'a> {
        todo!()
    }
}
