//! Binary Concrete Syntax Tree - convenient representation of tree_sitter `Node`s

use crate::backend::{
    data::{Data, DATA_NIL},
    metadata::{Metadata, META_CONS},
    rcst::{List, RCSTree},
    diff::Diff,
    patch::PatchError,
};
use std::{collections::HashMap, rc::Rc};

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
pub(crate) enum BCSTree<'a> {
    Leaf(Data<'a>),
    Node(Metadata, Rc<BCSTree<'a>>, Rc<BCSTree<'a>>),
}

impl<'a> BCSTree<'a> {
    pub(crate) fn size(&self) -> usize {
	match self {
	    Self::Leaf(_) => 1,
	    Self::Node(_, x, y) => x.size() + y.size(),
	}
    }
}

type DiffMem<'a> = HashMap<(Rc<BCSTree<'a>>, Rc<BCSTree<'a>>), Rc<Diff<'a>>>;

fn diff_leaf<'a>(x: Data<'a>, y: Data<'a>) -> Option<Diff<'a>> {
    match (x.named, y.named) {
	(false, false) => Some(Diff::RMod(y.range, y.text)),
	(true, true) => {
	    if x.node_type != y.node_type {
		None
	    } else if x.range != y.range || x.text != y.text {
		    Some(Diff::RMod(y.range, y.text))
	    } else {
		Some(Diff::Eps)
	    }
	}
	_ => None,
    }
}

pub(crate) fn diff<'a>(left: Rc<BCSTree<'a>>, right: Rc<BCSTree<'a>>, mem: &mut DiffMem<'a>) -> Rc<Diff<'a>> {
    if let Some(d) = mem.get(&(left.clone(), right.clone())) {
	d.clone()
    } else {
	let d = Rc::new(match (left.clone().as_ref(), right.clone().as_ref()) {

	    (BCSTree::Leaf(x), BCSTree::Leaf(y)) => diff_leaf(x.clone(), y.clone()).or(Some(Diff::Mod(left.clone(), right.clone()))).unwrap(),
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

pub(crate) fn patch<'a>(t: Rc<BCSTree<'a>>, d: Rc<Diff<'a>>) -> Result<Rc<BCSTree<'a>>, PatchError<'a>> {
    match (t.as_ref(), d.as_ref()) {
	(_, Diff::Eps) => Ok(t),
	(_, Diff::Mod(x, y)) if &t == x => Ok(y.clone()),
	(BCSTree::Node(t, x, y), Diff::TEps(td, dx, dy)) if t == td => {
	    let px = patch(x.clone(), dx.clone())?;
	    let py = patch(y.clone(), dy.clone())?;

	    Ok(Rc::new(BCSTree::Node(*t, px, py)))
	}
	(_, Diff::AddL(td, x, dy)) => patch(t, dy.clone()).map(|y| Rc::new(BCSTree::Node(*td, x.clone(), y))),
	(_, Diff::AddR(td, dx, y)) => patch(t, dx.clone()).map(|x| Rc::new(BCSTree::Node(*td, x, y.clone()))),
	(BCSTree::Node(_, _, y), Diff::DelL(dy)) => patch(y.clone(), dy.clone()),
	(BCSTree::Node(_, x, _), Diff::DelR(dx)) => patch(x.clone(), dx.clone()),
	(BCSTree::Node(t, x, y), Diff::TMod(t0, t1, dx, dy)) if t0 == t => {
	    let px = patch(x.clone(), dx.clone())?;
	    let py = patch(y.clone(), dy.clone())?;

	    Ok(Rc::new(BCSTree::Node(*t1, px, py)))
	}
	_ => Err(PatchError(t, d)),
    }
}

pub(crate) const LEAF_NIL: BCSTree = BCSTree::Leaf(DATA_NIL);

impl<'a> From<RCSTree<'a>> for BCSTree<'a> {
    fn from(t: RCSTree<'a>) -> Self {
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
