use crate::backend::diff::Diff;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub(crate) struct MergeConflict<'a>(pub(crate) Rc<Diff<'a>>, pub(crate) Rc<Diff<'a>>);

fn unite<'a>(
    ml: Result<Rc<Diff<'a>>, Vec<MergeConflict<'a>>>,
    mr: Result<Rc<Diff<'a>>, Vec<MergeConflict<'a>>>,
) -> Result<(Rc<Diff<'a>>, Rc<Diff<'a>>), Vec<MergeConflict<'a>>> {
    match (ml, mr) {
        (Ok(ml), Ok(mr)) => Ok((ml, mr)),
        (Err(mut e1), Err(e2)) => {
            e1.extend(e2);

            Err(e1)
        }
        (Err(e1), _) => Err(e1),
        (_, Err(e2)) => Err(e2),
    }
}

pub(crate) fn merge<'a>(
    left: Rc<Diff<'a>>,
    right: Rc<Diff<'a>>,
) -> Result<Rc<Diff<'a>>, Vec<MergeConflict<'a>>> {
    match (left.as_ref(), right.as_ref()) {
        (Diff::Eps, _) => Ok(right),
        (_, Diff::Eps) => Ok(left),
        (Diff::TEps(i1, l1, r1), Diff::TEps(i2, l2, r2)) if i1 == i2 => {
            let ml = merge(l1.clone(), l2.clone());
            let mr = merge(r1.clone(), r2.clone());

            unite(ml, mr).map(|(ml, mr)| Rc::new(Diff::TEps(*i1, ml, mr)))
        }
        (Diff::TMod(i1, j1, l1, r1), Diff::TMod(i2, j2, l2, r2)) if i1 == i2 && j1 == j2 => {
            let ml = merge(l1.clone(), l2.clone());
            let mr = merge(r1.clone(), r2.clone());

            unite(ml, mr).map(|(ml, mr)| Rc::new(Diff::TMod(*i1, *j1, ml, mr)))
        }
        (Diff::TEps(i1, l1, r1), Diff::TMod(i2, j, l2, r2)) if i1 == i2 => {
            let ml = merge(l1.clone(), l2.clone());
            let mr = merge(r1.clone(), r2.clone());

            unite(ml, mr).map(|(ml, mr)| Rc::new(Diff::TMod(*i1, *j, ml, mr)))
        }
        (Diff::TMod(i2, j, l2, r2), Diff::TEps(i1, l1, r1)) if i1 == i2 => {
            let ml = merge(l1.clone(), l2.clone());
            let mr = merge(r1.clone(), r2.clone());

            unite(ml, mr).map(|(ml, mr)| Rc::new(Diff::TMod(*i1, *j, ml, mr)))
        }
        (Diff::TEps(_, _, _), Diff::AddL(j, t, d)) => Ok(Rc::new(Diff::AddL(
            *j,
            t.clone(),
            merge(left.clone(), d.clone())?,
        ))),
        (Diff::TEps(_, _, _), Diff::AddR(j, d, t)) => Ok(Rc::new(Diff::AddR(
            *j,
            merge(left.clone(), d.clone())?,
            t.clone(),
        ))),
        (Diff::AddL(j, t, d), Diff::TEps(_, _, _)) => Ok(Rc::new(Diff::AddL(
            *j,
            t.clone(),
            merge(left.clone(), d.clone())?,
        ))),
        (Diff::AddR(j, d, t), Diff::TEps(_, _, _)) => Ok(Rc::new(Diff::AddR(
            *j,
            merge(left.clone(), d.clone())?,
            t.clone(),
        ))),
        (Diff::TEps(_, _, r), Diff::DelL(d)) => {
            Ok(Rc::new(Diff::DelL(merge(r.clone(), d.clone())?)))
        }
        (Diff::TEps(_, l, _), Diff::DelR(d)) => {
            Ok(Rc::new(Diff::DelR(merge(l.clone(), d.clone())?)))
        }
        (Diff::DelL(d), Diff::TEps(_, _, r)) => {
            Ok(Rc::new(Diff::DelL(merge(r.clone(), d.clone())?)))
        }
        (Diff::DelR(d), Diff::TEps(_, l, _)) => {
            Ok(Rc::new(Diff::DelR(merge(l.clone(), d.clone())?)))
        }
        (Diff::AddL(i1, t1, d1), Diff::AddL(i2, t2, d2)) if i1 == i2 && t1 == t2 => Ok(Rc::new(
            Diff::AddL(*i1, t1.clone(), merge(d1.clone(), d2.clone())?),
        )),
        (Diff::AddR(i1, d1, t1), Diff::AddR(i2, d2, t2)) if i1 == i2 && t1 == t2 => Ok(Rc::new(
            Diff::AddR(*i1, merge(d1.clone(), d2.clone())?, t1.clone()),
        )),
        (Diff::DelL(d1), Diff::DelL(d2)) => Ok(Rc::new(Diff::DelL(merge(d1.clone(), d2.clone())?))),
        (Diff::DelR(d1), Diff::DelR(d2)) => Ok(Rc::new(Diff::DelR(merge(d1.clone(), d2.clone())?))),
        _ if left == right => Ok(left),
        _ => Err(vec![MergeConflict(left, right)]),
    }
}
