//! Diff merging algorithm and conflict handling

use crate::backend::diff::Diff;
use std::rc::Rc;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub(crate) struct MergeConflict<'a>(pub(crate) Rc<Diff<'a>>, pub(crate) Rc<Diff<'a>>);

pub(crate) fn merge<'a>(
    left: Rc<Diff<'a>>,
    right: Rc<Diff<'a>>,
    conflicts: &mut Vec<MergeConflict<'a>>,
) -> Rc<Diff<'a>> {
    match (left.as_ref(), right.as_ref()) {
        (Diff::Eps, _) => right,
        (_, Diff::Eps) => left,
        (Diff::TEps(i1, l1, r1), Diff::TEps(i2, l2, r2)) if i1 == i2 => {
            let ml = merge(l1.clone(), l2.clone(), conflicts);
            let mr = merge(r1.clone(), r2.clone(), conflicts);

            Rc::new(Diff::TEps(*i1, ml, mr))
        }
        (Diff::TMod(i1, j1, l1, r1), Diff::TMod(i2, j2, l2, r2)) if i1 == i2 && j1 == j2 => {
            let ml = merge(l1.clone(), l2.clone(), conflicts);
            let mr = merge(r1.clone(), r2.clone(), conflicts);

            Rc::new(Diff::TMod(*i1, *j1, ml, mr))
        }
        (Diff::TEps(i1, l1, r1), Diff::TMod(i2, j, l2, r2)) if i1 == i2 => {
            let ml = merge(l1.clone(), l2.clone(), conflicts);
            let mr = merge(r1.clone(), r2.clone(), conflicts);

            Rc::new(Diff::TMod(*i1, *j, ml, mr))
        }
        (Diff::TMod(i2, j, l2, r2), Diff::TEps(i1, l1, r1)) if i1 == i2 => {
            let ml = merge(l1.clone(), l2.clone(), conflicts);
            let mr = merge(r1.clone(), r2.clone(), conflicts);

            Rc::new(Diff::TMod(*i1, *j, ml, mr))
        }
        (Diff::TEps(_, _, _), Diff::AddL(j, t, d)) => Rc::new(Diff::AddL(
            *j,
            t.clone(),
            merge(left.clone(), d.clone(), conflicts),
        )),
        (Diff::TEps(_, _, _), Diff::AddR(j, d, t)) => Rc::new(Diff::AddR(
            *j,
            merge(left.clone(), d.clone(), conflicts),
            t.clone(),
        )),
        (Diff::AddL(j, t, d), Diff::TEps(_, _, _)) => Rc::new(Diff::AddL(
            *j,
            t.clone(),
            merge(left.clone(), d.clone(), conflicts),
        )),
        (Diff::AddR(j, d, t), Diff::TEps(_, _, _)) => Rc::new(Diff::AddR(
            *j,
            merge(left.clone(), d.clone(), conflicts),
            t.clone(),
        )),
        (Diff::TEps(_, _, r), Diff::DelL(d)) => {
            Rc::new(Diff::DelL(merge(r.clone(), d.clone(), conflicts)))
        }
        (Diff::TEps(_, l, _), Diff::DelR(d)) => {
            Rc::new(Diff::DelR(merge(l.clone(), d.clone(), conflicts)))
        }
        (Diff::DelL(d), Diff::TEps(_, _, r)) => {
            Rc::new(Diff::DelL(merge(r.clone(), d.clone(), conflicts)))
        }
        (Diff::DelR(d), Diff::TEps(_, l, _)) => {
            Rc::new(Diff::DelR(merge(l.clone(), d.clone(), conflicts)))
        }
        (Diff::AddL(i1, t1, d1), Diff::AddL(i2, t2, d2)) if i1 == i2 && t1 == t2 => Rc::new(
            Diff::AddL(*i1, t1.clone(), merge(d1.clone(), d2.clone(), conflicts)),
        ),
        (Diff::AddR(i1, d1, t1), Diff::AddR(i2, d2, t2)) if i1 == i2 && t1 == t2 => Rc::new(
            Diff::AddR(*i1, merge(d1.clone(), d2.clone(), conflicts), t1.clone()),
        ),
        (Diff::DelL(d1), Diff::DelL(d2)) => {
            Rc::new(Diff::DelL(merge(d1.clone(), d2.clone(), conflicts)))
        }
        (Diff::DelR(d1), Diff::DelR(d2)) => {
            Rc::new(Diff::DelR(merge(d1.clone(), d2.clone(), conflicts)))
        }
        _ if left == right => left,
        (Diff::Err(_), _) => unreachable!(),
        (_, Diff::Err(_)) => unreachable!(),
        _ => {
            let c = MergeConflict(left, right);
            conflicts.push(c.clone());

            Rc::new(Diff::Err(c))
        }
    }
}

#[cfg(test)]
mod test {
    use super::merge;
    use crate::backend::{
        bcst::{bcst_to_code, diff_wrapper, patch, BCSTree},
        rcst::RCSTree,
    };
    use std::rc::Rc;
    use tree_sitter::Parser;

    #[test]
    fn merge0() {
        let base = "pub fn foo() { 5 + 6 }";
        let left = "pub fn foo() { 5 + 7 }";
        let right = "pub fn foo() { 5 - 6 }";
        let res = "pub fn foo() { 5 - 7 }";

        let mut parser = Parser::new();

        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();

        let btree = parser.parse(base, None).unwrap();
        let bnode = btree.root_node();

        let ltree = parser.parse(left, None).unwrap();
        let lnode = ltree.root_node();

        let rtree = parser.parse(right, None).unwrap();
        let rnode = rtree.root_node();

        let stree = parser.parse(res, None).unwrap();
        let snode = stree.root_node();

        let brcst = RCSTree::from(bnode, base);
        let bbcst: (BCSTree, usize) = brcst.into();
        let bbcst = (Rc::new(bbcst.0), bbcst.1);

        let lrcst = RCSTree::from(lnode, left);
        let lbcst: (BCSTree, usize) = lrcst.into();
        let lbcst = (Rc::new(lbcst.0), lbcst.1);

        let rrcst = RCSTree::from(rnode, right);
        let rbcst: (BCSTree, usize) = rrcst.into();
        let rbcst = (Rc::new(rbcst.0), rbcst.1);

        let srcst = RCSTree::from(snode, res);
        let sbcst: (BCSTree, usize) = srcst.into();
        let sbcst = (Rc::new(sbcst.0), sbcst.1);

        let diff_bl = diff_wrapper(bbcst.clone(), lbcst.clone());

        let diff_br = diff_wrapper(bbcst.clone(), rbcst.clone());

        let diff_bs = diff_wrapper(bbcst.clone(), sbcst.clone());

        let mut conflicts = Vec::new();

        let diff_m = merge(diff_bl, diff_br, &mut conflicts);

        assert!(conflicts.is_empty());

        assert_eq!(diff_bs, diff_m);
    }

    #[test]
    fn merge1() {
        let base = "pub fn foo() { 5 + 6 }";
        let left = "pub fn foo() { 5 + 7 }";
        let right = "pub fn foo() { 5 + 8 }";

        let mut parser = Parser::new();

        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();

        let btree = parser.parse(base, None).unwrap();
        let bnode = btree.root_node();

        let ltree = parser.parse(left, None).unwrap();
        let lnode = ltree.root_node();

        let rtree = parser.parse(right, None).unwrap();
        let rnode = rtree.root_node();

        let brcst = RCSTree::from(bnode, base);
        let bbcst: (BCSTree, usize) = brcst.into();
        let bbcst = (Rc::new(bbcst.0), bbcst.1);

        let lrcst = RCSTree::from(lnode, left);
        let lbcst: (BCSTree, usize) = lrcst.into();
        let lbcst = (Rc::new(lbcst.0), lbcst.1);

        let rrcst = RCSTree::from(rnode, right);
        let rbcst: (BCSTree, usize) = rrcst.into();
        let rbcst = (Rc::new(rbcst.0), rbcst.1);

        let diff_bl = diff_wrapper(bbcst.clone(), lbcst.clone());

        let diff_br = diff_wrapper(bbcst.clone(), rbcst.clone());

        let mut conflicts = Vec::new();

        merge(diff_bl, diff_br, &mut conflicts);

        assert_eq!(conflicts.len(), 1);
    }
}
