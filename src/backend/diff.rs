use crate::backend::{metadata::Metadata, bcst::BCSTree};
use std::{cmp::Ordering, rc::Rc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Diff {
    Eps,
    TEps(Metadata, Rc<Diff>, Rc<Diff>),
    Mod(Rc<BCSTree>, Rc<BCSTree>),
    TMod(Metadata, Metadata, Rc<Diff>, Rc<Diff>),
    AddL(Metadata, Rc<BCSTree>, Rc<Diff>),
    AddR(Metadata, Rc<Diff>, Rc<BCSTree>),
    DelL(Rc<Diff>),
    DelR(Rc<Diff>),
}

impl Diff {
    pub(crate) fn weight(&self) -> usize {
	match self {
	    Self::Eps => 0,
	    Self::TEps(_, x, y) => x.weight() + y.weight(),
	    Self::Mod(x, y) => 1 + x.size() + y.size(),
	    Self::TMod(_, _, x, y) => 1 + x.weight() + y.weight(),
	    Self::AddL(_, t, d) => 1 + t.size() + d.weight(),
	    Self::AddR(_, d, t) => 1 + t.size() + d.weight(),
	    Self::DelL(d) => 1 + d.weight(),
	    Self::DelR(d) => 1 + d.weight(),
	}
    }
}

impl PartialOrd for Diff {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
	Some(self.cmp(other))
    }
}

impl Ord for Diff {
    fn cmp(&self, other: &Self) -> Ordering {
	self.weight().cmp(&other.weight())
    }
}
