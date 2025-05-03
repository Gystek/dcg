//! Binary Concrete Syntax Tree - convenient representation of tree_sitter `Node`s

use crate::backend::{
    data::{Data, DATA_NIL},
    diff::Diff,
    metadata::{Metadata, META_CONS},
    patch::PatchError,
    rcst::{List, RCSTree},
};
use std::{collections::HashMap, rc::Rc};

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
pub(crate) enum BCSTree<'a> {
    Leaf(Data<'a>),
    Node(Metadata, Rc<BCSTree<'a>>, Rc<BCSTree<'a>>),
}

impl BCSTree<'_> {
    pub(crate) fn size(&self) -> usize {
        match self {
            Self::Leaf(_) => 1,
            Self::Node(_, x, y) => x.size() + y.size(),
        }
    }
}

pub(crate) fn bcst_to_code(t: Rc<BCSTree<'_>>) -> String {
    let mut s = String::new();

    bcst_to_code_rec(t, &mut s, (0, 0));

    s
}

fn bcst_to_code_rec(
    t: Rc<BCSTree<'_>>,
    s: &mut String,
    (mut line, mut col): (usize, usize),
) -> (usize, usize) {
    match t.as_ref() {
        BCSTree::Leaf(x) => {
            if x == &DATA_NIL {
                return (line, col);
            }

            let (xl, xc) = x.range.start;

            while line < xl {
                s.push('\n');
                line += 1;
                col = 0;
            }

            while col < xc {
                s.push(' ');
                col += 1;
            }

            s.push_str(x.text);

            x.range.end
        }
        BCSTree::Node(_, left, right) => {
            let npos = bcst_to_code_rec(left.clone(), s, (line, col));
            bcst_to_code_rec(right.clone(), s, npos)
        }
    }
}

type DiffMem<'a> = HashMap<(Rc<BCSTree<'a>>, Rc<BCSTree<'a>>), Rc<Diff<'a>>>;

fn diff_leaf<'a>(x: Data<'a>, y: Data<'a>) -> Option<Diff<'a>> {
    match (x.named, y.named) {
        (false, false) => {
            if y.node_type == x.node_type && y.range == x.range && y.text == x.text {
                Some(Diff::Eps)
            } else {
                Some(Diff::RMod(y.node_type, y.range, y.byte_range, y.text))
            }
        }
        (true, true) => {
            if x.node_type != y.node_type {
                None
            } else if x.range != y.range || x.text != y.text {
                Some(Diff::RMod(y.node_type, y.range, y.byte_range, y.text))
            } else {
                Some(Diff::Eps)
            }
        }
        _ => None,
    }
}

pub(crate) fn diff<'a>(
    left: Rc<BCSTree<'a>>,
    right: Rc<BCSTree<'a>>,
    mem: &mut DiffMem<'a>,
) -> Rc<Diff<'a>> {
    if let Some(d) = mem.get(&(left.clone(), right.clone())) {
        d.clone()
    } else {
        let d = Rc::new(match (left.clone().as_ref(), right.clone().as_ref()) {
            (BCSTree::Leaf(x), BCSTree::Leaf(y)) => {
                diff_leaf(x.clone(), y.clone()).unwrap_or(Diff::Mod(left.clone(), right.clone()))
            }
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
                }
                .unwrap()
            }
            (BCSTree::Leaf(_), BCSTree::Node(t, x, y)) => {
                let dly = diff(left.clone(), y.clone(), mem);
                let dlx = diff(left.clone(), x.clone(), mem);

                [
                    Diff::Mod(left.clone(), right.clone()),
                    Diff::AddL(*t, x.clone(), dly),
                    Diff::AddR(*t, dlx, y.clone()),
                ]
                .into_iter()
                .min()
                .unwrap()
            }
            (BCSTree::Node(_, x, y), BCSTree::Leaf(_)) => {
                let dyr = diff(y.clone(), right.clone(), mem);
                let dxr = diff(x.clone(), right.clone(), mem);

                [
                    Diff::Mod(left.clone(), right.clone()),
                    Diff::DelL(dyr),
                    Diff::DelR(dxr),
                ]
                .into_iter()
                .min()
                .unwrap()
            }
        });

        mem.insert((left, right), d.clone());

        d
    }
}

pub(crate) fn patch<'a>(
    t: Rc<BCSTree<'a>>,
    d: Rc<Diff<'a>>,
) -> Result<Rc<BCSTree<'a>>, PatchError<'a>> {
    match (t.as_ref(), d.as_ref()) {
        (_, Diff::Err(_)) => unreachable!(),
        (_, Diff::Eps) => Ok(t),
        (BCSTree::Leaf(x), Diff::RMod(t, r, br, txt)) => {
            let nx = Data {
                node_type: *t,
                range: r.clone(),
                byte_range: br.clone(),
                text: txt,
                named: x.named,
            };

            Ok(Rc::new(BCSTree::Leaf(nx)))
        }
        (_, Diff::Mod(x, y)) if &t == x => Ok(y.clone()),
        (BCSTree::Node(t, x, y), Diff::TEps(td, dx, dy)) if t == td => {
            let px = patch(x.clone(), dx.clone())?;
            let py = patch(y.clone(), dy.clone())?;

            Ok(Rc::new(BCSTree::Node(*t, px, py)))
        }
        (_, Diff::AddL(td, x, dy)) => {
            patch(t, dy.clone()).map(|y| Rc::new(BCSTree::Node(*td, x.clone(), y)))
        }
        (_, Diff::AddR(td, dx, y)) => {
            patch(t, dx.clone()).map(|x| Rc::new(BCSTree::Node(*td, x, y.clone())))
        }
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
                        xs => {
                            let bx = x.as_ref().clone().into();
                            let bxs = RCSTree::Node(META_CONS, xs.clone()).into();

                            BCSTree::Node(m, Rc::new(bx), Rc::new(bxs))
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

#[cfg(test)]
mod test {
    use tree_sitter::Parser;

    use std::{collections::HashMap, rc::Rc};

    use crate::backend::diff::ered;

    use super::{diff, patch, BCSTree, RCSTree};

    #[test]
    fn no_difference() {
        let code = "pub fn foo() {\n  1\n}";

        let mut parser = Parser::new();

        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(code, None).unwrap();
        let node = tree.root_node();

        let rcst = RCSTree::from(node, code);
        let bcst: Rc<BCSTree> = Rc::new(rcst.into());

        let mut mem = HashMap::new();
        let diff = diff(bcst.clone(), bcst.clone(), &mut mem);

        let patch = patch(bcst.clone(), diff).unwrap();

        assert_eq!(bcst, patch)
    }

    #[test]
    fn diff0() {
        let left = "pub fn foo() {\n  1\n}";
        let right = "pub fn bar() {\n  1\n}";

        let mut parser = Parser::new();

        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();

        let ltree = parser.parse(left, None).unwrap();
        let lnode = ltree.root_node();

        let rtree = parser.parse(right, None).unwrap();
        let rnode = rtree.root_node();

        let lrcst = RCSTree::from(lnode, left);
        let lbcst: Rc<BCSTree> = Rc::new(lrcst.into());

        let rrcst = RCSTree::from(rnode, right);
        let rbcst: Rc<BCSTree> = Rc::new(rrcst.into());

        let mut mem = HashMap::new();
        let diff = diff(lbcst.clone(), rbcst.clone(), &mut mem);

        let patch = patch(lbcst, diff).unwrap();

        assert_eq!(rbcst, patch)
    }

    #[test]
    fn diff1() {
        let left = "pub fn foo() {\n  1\n}";
        let right = "pub fn foo() {\nlet x = 5;\n  3\n}";

        let mut parser = Parser::new();

        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();

        let ltree = parser.parse(left, None).unwrap();
        let lnode = ltree.root_node();

        let rtree = parser.parse(right, None).unwrap();
        let rnode = rtree.root_node();

        let lrcst = RCSTree::from(lnode, left);
        let lbcst: Rc<BCSTree> = Rc::new(lrcst.into());

        let rrcst = RCSTree::from(rnode, right);
        let rbcst: Rc<BCSTree> = Rc::new(rrcst.into());

        let mut mem = HashMap::new();
        let diff = diff(lbcst.clone(), rbcst.clone(), &mut mem);

        let patch = patch(lbcst, diff).unwrap();

        assert_eq!(rbcst, patch)
    }

    #[test]
    fn diff2() {
        let left = "pub fn foo() {\n  1\n}";
        let right = "struct Foo { i: i32 }";

        let mut parser = Parser::new();

        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();

        let ltree = parser.parse(left, None).unwrap();
        let lnode = ltree.root_node();

        let rtree = parser.parse(right, None).unwrap();
        let rnode = rtree.root_node();

        let lrcst = RCSTree::from(lnode, left);
        let lbcst: Rc<BCSTree> = Rc::new(lrcst.into());

        let rrcst = RCSTree::from(rnode, right);
        let rbcst: Rc<BCSTree> = Rc::new(rrcst.into());

        let mut mem = HashMap::new();
        let diff = diff(lbcst.clone(), rbcst.clone(), &mut mem);

        let patch = patch(lbcst, diff).unwrap();

        assert_eq!(rbcst, patch);
    }

    #[test]
    fn bcst_to_code() {
        let code = "pub fn foo() {\n  1\n}";

        let mut parser = Parser::new();

        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(code, None).unwrap();
        let node = tree.root_node();

        let rcst = RCSTree::from(node, code);
        let bcst: Rc<BCSTree> = Rc::new(rcst.into());

        let stred = super::bcst_to_code(bcst);

        assert_eq!(code, &stred)
    }

    #[test]
    fn ered0() {
        let left = "pub fn foo() {\n  1\n}";
        let right = "pub fn bar() {\n  1\n}";

        let mut parser = Parser::new();

        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();

        let ltree = parser.parse(left, None).unwrap();
        let lnode = ltree.root_node();

        let rtree = parser.parse(right, None).unwrap();
        let rnode = rtree.root_node();

        let lrcst = RCSTree::from(lnode, left);
        let lbcst: Rc<BCSTree> = Rc::new(lrcst.into());

        let rrcst = RCSTree::from(rnode, right);
        let rbcst: Rc<BCSTree> = Rc::new(rrcst.into());

        let mut mem = HashMap::new();
        let diff0 = diff(lbcst.clone(), rbcst.clone(), &mut mem);

        let patch0 = patch(lbcst.clone(), diff0.clone()).unwrap();

        let diff1 = ered(diff0);

        let patch1 = patch(lbcst, diff1).unwrap();

        assert_eq!(patch0, patch1)
    }

    #[test]
    fn ered1() {
        let left = "pub fn foo() {\n  1\n}";
        let right = "pub fn foo() {\nlet x = 5;\n  3\n}";

        let mut parser = Parser::new();

        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();

        let ltree = parser.parse(left, None).unwrap();
        let lnode = ltree.root_node();

        let rtree = parser.parse(right, None).unwrap();
        let rnode = rtree.root_node();

        let lrcst = RCSTree::from(lnode, left);
        let lbcst: Rc<BCSTree> = Rc::new(lrcst.into());

        let rrcst = RCSTree::from(rnode, right);
        let rbcst: Rc<BCSTree> = Rc::new(rrcst.into());

        let mut mem = HashMap::new();
        let diff0 = diff(lbcst.clone(), rbcst.clone(), &mut mem);

        let patch0 = patch(lbcst.clone(), diff0.clone()).unwrap();

        let diff1 = ered(diff0);

        let patch1 = patch(lbcst, diff1).unwrap();

        assert_eq!(patch0, patch1)
    }
}
