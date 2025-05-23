//! Tree differences datatype
use crate::backend::{bcst::Twh, merge::MergeConflict, metadata::Metadata};
use std::{cmp::Ordering, ops::Range, rc::Rc};

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub(crate) enum Diff<'a> {
    Eps,
    Err(MergeConflict<'a>), /* exclusively for internal use by the merge error handling algorithm */
    RMod(Option<u16>, Range<(usize, usize)>, Range<usize>, &'a str),
    TEps(Metadata, Rc<Diff<'a>>, Rc<Diff<'a>>),
    Mod(Twh<'a>, Twh<'a>),
    TMod(Metadata, Metadata, Rc<Diff<'a>>, Rc<Diff<'a>>),
    AddL(Metadata, Twh<'a>, Rc<Diff<'a>>),
    AddR(Metadata, Rc<Diff<'a>>, Twh<'a>),
    DelL(Rc<Diff<'a>>),
    DelR(Rc<Diff<'a>>),
}

impl Diff<'_> {
    pub(crate) fn is_eps(&self) -> bool {
        self == &Diff::Eps
    }
    pub(crate) fn weight(&self) -> usize {
        match self {
            Self::Eps => 0,
            Self::Err(_) => 0,
            Self::RMod(_, _, _, _) => 0,
            Self::TEps(_, x, y) => x.weight() + y.weight(),
            Self::Mod((x, _), (y, _)) => 1 + x.size() + y.size(),
            Self::TMod(_, _, x, y) => 1 + x.weight() + y.weight(),
            Self::AddL(_, (t, _), d) => 1 + t.size() + d.weight(),
            Self::AddR(_, d, (t, _)) => 1 + t.size() + d.weight(),
            Self::DelL(d) => 1 + d.weight(),
            Self::DelR(d) => 1 + d.weight(),
        }
    }
}

pub(crate) fn ered<'a>(d: Rc<Diff<'a>>) -> Rc<Diff<'a>> {
    match d.as_ref() {
        Diff::Mod(x, y) if x == y => Rc::new(Diff::Eps),
        Diff::Eps | Diff::Err(_) | Diff::Mod(_, _) | Diff::RMod(_, _, _, _) => d,
        Diff::TEps(a, x, y) => {
            let ex = ered(x.clone());
            let ey = ered(y.clone());

            if ex.is_eps() && ey.is_eps() {
                Rc::new(Diff::Eps)
            } else if &ex == x && &ey == y {
                d
            } else {
                Rc::new(Diff::TEps(*a, ex, ey))
            }
        }
        Diff::TMod(i, j, x, y) => {
            let ex = ered(x.clone());
            let ey = ered(y.clone());

            if &ex == x && &ey == y {
                d
            } else {
                Rc::new(Diff::TMod(*i, *j, ex, ey))
            }
        }
        Diff::AddL(i, t, dl) => {
            let edl = ered(dl.clone());

            if &edl == dl {
                d
            } else {
                Rc::new(Diff::AddL(*i, t.clone(), edl))
            }
        }
        Diff::AddR(i, dl, t) => {
            let edl = ered(dl.clone());

            if &edl == dl {
                d
            } else {
                Rc::new(Diff::AddR(*i, edl, t.clone()))
            }
        }
        Diff::DelL(dl) => {
            let edl = ered(dl.clone());

            if &edl == dl {
                d
            } else {
                Rc::new(Diff::DelL(edl))
            }
        }
        Diff::DelR(dl) => {
            let edl = ered(dl.clone());

            if &edl == dl {
                d
            } else {
                Rc::new(Diff::DelR(edl))
            }
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
