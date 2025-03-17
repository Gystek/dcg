use crate::backend::{
    data::{Data, DATA_NIL},
    metadata::{Metadata, META_CONS},
    rcst::{List, RCSTree},
    diff::Diff,
};
use std::{collections::HashMap, rc::Rc, sync::Once};

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
pub(crate) enum BCSTree {
    Leaf(Data),
    Node(Metadata, Rc<BCSTree>, Rc<BCSTree>),
}

impl BCSTree {
    pub(crate) fn size(&self) -> usize {
	match self {
	    Self::Leaf(_) => 1,
	    Self::Node(_, x, y) => x.size() + y.size(),
	}
    }
}

type DiffMem = HashMap<(Rc<BCSTree>, Rc<BCSTree>), Rc<Diff>>;

pub(crate) fn diff(left: Rc<BCSTree>, right: Rc<BCSTree>, mem: &mut DiffMem) -> Rc<Diff> {
    if let Some(d) = mem.get(&(left.clone(), right.clone())) {
	d.clone()
    } else {
	let d = Rc::new(match (left.clone().as_ref(), right.clone().as_ref()) {
	    (BCSTree::Leaf(x), BCSTree::Leaf(y)) if x == y => Diff::Eps,
	    (BCSTree::Leaf(_), BCSTree::Leaf(_)) => Diff::Mod(left.clone(), right.clone()),
	    (BCSTree::Node(a, x0, y0), BCSTree::Node(b, x1, y1)) => {
		let dxx = diff(x0.clone(), x1.clone(), mem);
		let dyy = diff(y0.clone(), y1.clone(), mem);
		let dly = diff(left.clone(), y1.clone(), mem);
		let dlx = diff(left.clone(), x1.clone(), mem);
		let dyr = diff(y0.clone(), right.clone(), mem);
		let dxr = diff(x0.clone(), right.clone(), mem);

		let di = Diff::TEps(*a, dxx.clone(), dyy.clone());
		let dm = Diff::Mod(left.clone(), right.clone());
		let dtm = Diff::TMod(*a, *b, dxx, dyy);
		let dal = Diff::AddL(*b, x1.clone(), dly);
		let dar = Diff::AddR(*b, dlx, y1.clone());
		let ddl = Diff::DelL(dyr);
		let ddr = Diff::DelR(dxr);

		if a == b {
		    [di, dal, dar, ddl, ddr].into_iter().min()
		} else {
		    [dm, dtm, dal, dar, ddl, ddr].into_iter().min()
		}.unwrap()
	    }
	    (BCSTree::Leaf(_), BCSTree::Node(t, x, y)) => {
		let dly = diff(left.clone(), y.clone(), mem);
		let dlx = diff(left.clone(), x.clone(), mem);

		[Diff::Mod(left.clone(), right.clone()),
		 Diff::AddL(*t, x.clone(), dly),
		 Diff::AddR(*t, dlx, y.clone())].into_iter().min().unwrap()
	    }
	    (BCSTree::Node(_, x, y), BCSTree::Leaf(_)) => {
		let dyr = diff(y.clone(), right.clone(), mem);
		let dxr = diff(x.clone(), right.clone(), mem);

		[Diff::Mod(left.clone(), right.clone()),
		 Diff::DelL(dyr),
		 Diff::DelR(dxr)].into_iter().min().unwrap()
	    }
	});

	mem.insert((left, right), d.clone());

	d
    }
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
