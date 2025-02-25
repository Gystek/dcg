//! Diff data type for representing changes between `BCSTree`s.
use std::cmp::Ordering;

use crate::backend::{types::{Data, Metadata}, bcst::BCSTree};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Diff<A: Data, B: Metadata> {
    Eps,
    TEps(B, Box<Diff<A, B>>, Box<Diff<A, B>>),
    Mod(BCSTree<A, B>, BCSTree<A, B>),
    TMod(B, B, Box<Diff<A, B>>, Box<Diff<A, B>>),
    AddL(B, BCSTree<A, B>, Box<Diff<A, B>>),
    AddR(B, Box<Diff<A, B>>, BCSTree<A, B>),
    DelL(Box<Diff<A, B>>),
    DelR(Box<Diff<A, B>>),
}

impl<A: Data, B: Metadata> Diff<A, B> {
    /// The weight of a `Diff` represents its cost of application
    /// and of storage (once serialised).
    pub(crate) fn weight(&self) -> usize {
	match self {
	    Self::Eps => 0,
	    Self::TEps(_, left, right) => left.weight() + right.weight(),
	    Self::Mod(from, to) => 1 + from.size() + to.size(),
	    Self::TMod(_, _, left, right) => 1 + left.weight() + right.weight(),
	    Self::AddL(_, t, d) => 1 + t.size() + d.weight(),
	    Self::AddR(_, d, t) => 1 + t.size() + d.weight(),
	    Self::DelL(t) => 1 + t.weight(),
	    Self::DelR(t) => 1 + t.weight(),
	}
    }
}

impl<A: Data, B: Metadata> PartialOrd for Diff<A, B> {
    fn partial_cmp(&self, other: &Diff<A, B>) -> Option<Ordering> {
	Some(self.cmp(other))
    }
}

impl<A: Data, B: Metadata> Ord for Diff<A, B> {
    fn cmp(&self, other: &Diff<A, B>) -> Ordering {
	self.weight().cmp(&other.weight())
    }
}
