use crate::backend::{
    data::{Data, DATA_NIL},
    metadata::{Metadata, META_CONS},
    rcst::{List, RCSTree},
};
use std::{rc::Rc, sync::Once};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BCSTree {
    Leaf(Data),
    Node(Metadata, Rc<BCSTree>, Rc<BCSTree>),
}

pub(crate) const LEAF_NIL: BCSTree = BCSTree::Leaf(DATA_NIL);

impl From<RCSTree> for BCSTree {
    fn from(t: RCSTree) -> Self {
        match t {
            RCSTree::Leaf(x) => BCSTree::Leaf(x),
            RCSTree::Node(m, xs) => {
                if let Some(x) = xs.car() {
                    /* unwrap is safe */
                    match xs.cdr().unwrap().as_ref() {
                        List::Nil => {
                            let bx = x.as_ref().clone().into();

                            BCSTree::Node(m, Rc::new(bx), Rc::new(LEAF_NIL))
                        }
                        List::Cons(y, ys) => {
                            let by = y.as_ref().clone().into();
                            let bys = RCSTree::Node(META_CONS, ys.as_ref().clone()).into();

                            BCSTree::Node(m, Rc::new(by), Rc::new(bys))
                        }
                    }
                } else {
                    let nil_rc = Rc::new(LEAF_NIL);
                    BCSTree::Node(m, nil_rc.clone(), nil_rc)
                }
            }
        }
    }
}
