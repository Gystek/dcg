//! Tree differences datatype
use crate::backend::{bcst::BCSTree, merge::MergeConflict, metadata::Metadata};
use std::{cmp::Ordering, ops::Range, rc::Rc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Diff<'a> {
    Eps,
    Err(MergeConflict<'a>), /* exclusively for internal use by the merge error handling algorithm */
    RMod(Option<u16>, Range<(usize, usize)>, Range<usize>, &'a str),
    TEps(Metadata, Rc<Diff<'a>>, Rc<Diff<'a>>),
    Mod(Rc<BCSTree<'a>>, Rc<BCSTree<'a>>),
    TMod(Metadata, Metadata, Rc<Diff<'a>>, Rc<Diff<'a>>),
    AddL(Metadata, Rc<BCSTree<'a>>, Rc<Diff<'a>>),
    AddR(Metadata, Rc<Diff<'a>>, Rc<BCSTree<'a>>),
    DelL(Rc<Diff<'a>>),
    DelR(Rc<Diff<'a>>),
}

impl Diff<'_> {
    pub(crate) fn weight(&self) -> usize {
        match self {
            Self::Eps => 0,
            Self::Err(_) => 0,
            Self::RMod(_, _, _, _) => 0,
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

impl PartialOrd for Diff<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Diff<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.weight().cmp(&other.weight())
    }
}
