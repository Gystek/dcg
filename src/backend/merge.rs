//! Diff merging algorithm and conflict handling

use crate::backend::diff::Diff;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MergeConflict<'a>(pub(crate) Rc<Diff<'a>>, pub(crate) Rc<Diff<'a>>);

fn extract_diff<'a>(ml: Result<Rc<Diff<'a>>, MergeConflict<'a>>) -> Rc<Diff<'a>> {
    match ml {
        Ok(x) => x,
        Err(e) => Rc::new(Diff::Err(e)),
    }
}

pub(crate) fn merge<'a>(
    left: Rc<Diff<'a>>,
    right: Rc<Diff<'a>>,
    conflicts: &mut Vec<MergeConflict<'a>>,
) -> Result<Rc<Diff<'a>>, MergeConflict<'a>> {
    match (left.as_ref(), right.as_ref()) {
        (Diff::Eps, _) => Ok(right),
        (_, Diff::Eps) => Ok(left),
        (Diff::TEps(i1, l1, r1), Diff::TEps(i2, l2, r2)) if i1 == i2 => {
            let ml = merge(l1.clone(), l2.clone(), conflicts);
            let mr = merge(r1.clone(), r2.clone(), conflicts);

            Ok(Rc::new(Diff::TEps(*i1, extract_diff(ml), extract_diff(mr))))
        }
        (Diff::TMod(i1, j1, l1, r1), Diff::TMod(i2, j2, l2, r2)) if i1 == i2 && j1 == j2 => {
            let ml = merge(l1.clone(), l2.clone(), conflicts);
            let mr = merge(r1.clone(), r2.clone(), conflicts);

            Ok(Rc::new(Diff::TMod(
                *i1,
                *j1,
                extract_diff(ml),
                extract_diff(mr),
            )))
        }
        (Diff::TEps(i1, l1, r1), Diff::TMod(i2, j, l2, r2)) if i1 == i2 => {
            let ml = merge(l1.clone(), l2.clone(), conflicts);
            let mr = merge(r1.clone(), r2.clone(), conflicts);

            Ok(Rc::new(Diff::TMod(
                *i1,
                *j,
                extract_diff(ml),
                extract_diff(mr),
            )))
        }
        (Diff::TMod(i2, j, l2, r2), Diff::TEps(i1, l1, r1)) if i1 == i2 => {
            let ml = merge(l1.clone(), l2.clone(), conflicts);
            let mr = merge(r1.clone(), r2.clone(), conflicts);

            Ok(Rc::new(Diff::TMod(
                *i1,
                *j,
                extract_diff(ml),
                extract_diff(mr),
            )))
        }
        (Diff::TEps(_, _, _), Diff::AddL(j, t, d)) => Ok(Rc::new(Diff::AddL(
            *j,
            t.clone(),
            merge(left.clone(), d.clone(), conflicts)?,
        ))),
        (Diff::TEps(_, _, _), Diff::AddR(j, d, t)) => Ok(Rc::new(Diff::AddR(
            *j,
            merge(left.clone(), d.clone(), conflicts)?,
            t.clone(),
        ))),
        (Diff::AddL(j, t, d), Diff::TEps(_, _, _)) => Ok(Rc::new(Diff::AddL(
            *j,
            t.clone(),
            merge(left.clone(), d.clone(), conflicts)?,
        ))),
        (Diff::AddR(j, d, t), Diff::TEps(_, _, _)) => Ok(Rc::new(Diff::AddR(
            *j,
            merge(left.clone(), d.clone(), conflicts)?,
            t.clone(),
        ))),
        (Diff::TEps(_, _, r), Diff::DelL(d)) => {
            Ok(Rc::new(Diff::DelL(merge(r.clone(), d.clone(), conflicts)?)))
        }
        (Diff::TEps(_, l, _), Diff::DelR(d)) => {
            Ok(Rc::new(Diff::DelR(merge(l.clone(), d.clone(), conflicts)?)))
        }
        (Diff::DelL(d), Diff::TEps(_, _, r)) => {
            Ok(Rc::new(Diff::DelL(merge(r.clone(), d.clone(), conflicts)?)))
        }
        (Diff::DelR(d), Diff::TEps(_, l, _)) => {
            Ok(Rc::new(Diff::DelR(merge(l.clone(), d.clone(), conflicts)?)))
        }
        (Diff::AddL(i1, t1, d1), Diff::AddL(i2, t2, d2)) if i1 == i2 && t1 == t2 => Ok(Rc::new(
            Diff::AddL(*i1, t1.clone(), merge(d1.clone(), d2.clone(), conflicts)?),
        )),
        (Diff::AddR(i1, d1, t1), Diff::AddR(i2, d2, t2)) if i1 == i2 && t1 == t2 => Ok(Rc::new(
            Diff::AddR(*i1, merge(d1.clone(), d2.clone(), conflicts)?, t1.clone()),
        )),
        (Diff::DelL(d1), Diff::DelL(d2)) => Ok(Rc::new(Diff::DelL(merge(
            d1.clone(),
            d2.clone(),
            conflicts,
        )?))),
        (Diff::DelR(d1), Diff::DelR(d2)) => Ok(Rc::new(Diff::DelR(merge(
            d1.clone(),
            d2.clone(),
            conflicts,
        )?))),
        _ if left == right => Ok(left),
        (Diff::Err(_), _) => unreachable!(),
        (_, Diff::Err(_)) => unreachable!(),
        _ => {
            let c = MergeConflict(left, right);
            conflicts.push(c.clone());

            Err(c)
        }
    }
}

#[cfg(test)]
mod test {}
