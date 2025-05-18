//! Binary Concrete Syntax Tree - convenient representation of tree_sitter `Node`s

use crate::backend::{
    data::{Data, DATA_NIL},
    diff::Diff,
    metadata::{Metadata, META_CONS},
    patch::PatchError,
    rcst::{List, RCSTree},
};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
    rc::Rc,
};

/* Tree With Height - to avoid redundant calculations */
pub(crate) type TWH<'a> = (Rc<BCSTree<'a>>, usize);

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
pub(crate) enum BCSTree<'a> {
    Leaf(Data<'a>),
    Node(Metadata, TWH<'a>, TWH<'a>),
}

impl<'a> Ord for BCSTree<'a> {
    /* bogus implementation - only needed for storage in the heap */
    fn cmp(&self, _: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl<'a> PartialOrd for BCSTree<'a> {
    /* bogus implementation - only needed for storage in the heap */
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> BCSTree<'a> {
    pub(crate) fn size(&self) -> usize {
        match self {
            Self::Leaf(_) => 1,
            Self::Node(_, (x, _), (y, _)) => x.size() + y.size(),
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
        BCSTree::Node(_, (left, _), (right, _)) => {
            let npos = bcst_to_code_rec(left.clone(), s, (line, col));
            bcst_to_code_rec(right.clone(), s, npos)
        }
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
pub(crate) enum FlatDiff {
    Eps,
    TEps(Metadata),
    Mod,
    RMod,
    TMod(Metadata, Metadata),
    AddL(Metadata),
    AddR(Metadata),
    DelL,
    DelR,
    Start,
}

#[derive(Clone, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GraphDiffValue<'a> {
    Final(Rc<Diff<'a>>),
    Next(TWH<'a>, TWH<'a>),
    NextLR((TWH<'a>, TWH<'a>), (TWH<'a>, TWH<'a>)),
}

pub(crate) type GDV<'a> = GraphDiffValue<'a>;

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub(crate) struct GraphDiff<'a>(pub(crate) FlatDiff, pub(crate) GraphDiffValue<'a>);

impl<'a> Ord for GraphDiff<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.cmp(&other.1)
    }
}

impl<'a> PartialOrd for GraphDiff<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn diff_leaf<'a>(x: Data<'a>, y: Data<'a>) -> Option<GraphDiff<'a>> {
    match (x.named, y.named) {
        (false, false) => {
            if y.node_type == x.node_type && y.range == x.range && y.text == x.text {
                Some(GraphDiff(FlatDiff::Eps, GDV::Final(Rc::new(Diff::Eps))))
            } else {
                Some(GraphDiff(
                    FlatDiff::Mod,
                    GDV::Final(Rc::new(Diff::RMod(
                        y.node_type,
                        y.range,
                        y.byte_range,
                        y.text,
                    ))),
                ))
            }
        }
        (true, true) => {
            if x.node_type != y.node_type {
                None
            } else if x.range != y.range || x.text != y.text {
                Some(GraphDiff(
                    FlatDiff::RMod,
                    GDV::Final(Rc::new(Diff::RMod(
                        y.node_type,
                        y.range,
                        y.byte_range,
                        y.text,
                    ))),
                ))
            } else {
                Some(GraphDiff(FlatDiff::Eps, GDV::Final(Rc::new(Diff::Eps))))
            }
        }
        _ => None,
    }
}

/// A* heuristic
fn h((_, lh): &TWH<'_>, (_, rh): &TWH<'_>) -> usize {
    (*lh).min(*rh)
}

fn hgd(gd: &GraphDiff<'_>) -> usize {
    match gd {
        GraphDiff(_, GDV::Final(_)) => 0,
        GraphDiff(_, GDV::Next(left, right)) => h(left, right),
        GraphDiff(_, GDV::NextLR((left0, right0), (left1, right1))) => {
            h(left0, right0).min(h(left1, right1))
        }
    }
}

fn cost(gd: &GraphDiff<'_>) -> usize {
    todo!()
}

fn reconstruct_diff<'a>(
    d: Rc<Diff<'a>>,
    gd: GraphDiff<'a>,
    parents: &HashMap<GraphDiff<'a>, GraphDiff<'a>>,
) -> Rc<Diff<'a>> {
    todo!()
}

pub(crate) fn diff_wrapper<'a>(left: TWH<'a>, right: TWH<'a>) -> Rc<Diff<'a>> {
    let mut parents = HashMap::new();
    let mut heap = BinaryHeap::new();

    let max_cost = left.1 + right.1;

    heap.push((max_cost, GraphDiff(FlatDiff::Start, GDV::Next(left, right))));

    diff(max_cost, &mut parents, &mut heap)
}

/// When this function is first called, the heap should contain (left, right) with the value max_cost */
fn diff<'a>(
    /* to emulate a min-heap. should be set to left.len() + right.len()
     * before first calling the function.
     */
    max_cost: usize,
    parents: &mut HashMap<GraphDiff<'a>, GraphDiff<'a>>,
    /* first `usize` is the MAX_COST - cost of the node, as BinaryHeap
     * is a max-heap.
     */
    heap: &mut BinaryHeap<(usize, GraphDiff<'a>)>,
) -> Rc<Diff<'a>> {
    while let Some((mf, gd)) = heap.pop() {
        let f = max_cost - mf;
        let g = f - hgd(&gd);

        match gd {
            GraphDiff(_, GDV::Final(ref d)) => return reconstruct_diff(d.clone(), gd, parents),
            GraphDiff(fd, GDV::Next((ref left, lh), (ref right, rh))) => {
                let neighbours = match (left.as_ref(), right.as_ref()) {
                    (BCSTree::Leaf(x), BCSTree::Leaf(y)) => {
                        vec![diff_leaf(x.clone(), y.clone()).unwrap_or(GraphDiff(
                            FlatDiff::Mod,
                            GDV::Final(Rc::new(Diff::Mod((left.clone(), lh), (right.clone(), rh)))),
                        ))]
                    }
                    (
                        BCSTree::Node(a, (x0, x0h), (y0, y0h)),
                        BCSTree::Node(b, (x1, x1h), (y1, y1h)),
                    ) => {
                        let mut cn = vec![
                            GraphDiff(
                                FlatDiff::AddL(*b),
                                GDV::Next((left.clone(), lh), (y1.clone(), *y1h)),
                            ),
                            GraphDiff(
                                FlatDiff::AddR(*b),
                                GDV::Next((left.clone(), lh), (x1.clone(), *x1h)),
                            ),
                            GraphDiff(
                                FlatDiff::DelL,
                                GDV::Next((y0.clone(), *y0h), (right.clone(), rh)),
                            ),
                            GraphDiff(
                                FlatDiff::DelR,
                                GDV::Next((x0.clone(), *x0h), (right.clone(), rh)),
                            ),
                        ];

                        let double = GDV::NextLR(
                            ((x0.clone(), *x0h), (x1.clone(), *x1h)),
                            ((y0.clone(), *y0h), (y1.clone(), *y1h)),
                        );

                        if a == b {
                            cn.push(GraphDiff(FlatDiff::TEps(*a), double));
                        } else {
                            cn.push(GraphDiff(FlatDiff::TMod(*a, *b), double));
                            cn.push(GraphDiff(
                                FlatDiff::Mod,
                                GDV::Final(Rc::new(Diff::Mod(
                                    (left.clone(), lh),
                                    (right.clone(), rh),
                                ))),
                            ));
                        }

                        cn
                    }
                    (BCSTree::Leaf(_), BCSTree::Node(t, x, y)) => {
                        vec![
                            GraphDiff(
                                FlatDiff::Mod,
                                GDV::Final(Rc::new(Diff::Mod(
                                    (left.clone(), lh),
                                    (right.clone(), rh),
                                ))),
                            ),
                            GraphDiff(FlatDiff::AddL(*t), GDV::Next((left.clone(), lh), y.clone())),
                            GraphDiff(FlatDiff::AddR(*t), GDV::Next((left.clone(), lh), x.clone())),
                        ]
                    }
                    (BCSTree::Node(_, x, y), BCSTree::Leaf(_)) => {
                        vec![
                            GraphDiff(
                                FlatDiff::Mod,
                                GDV::Final(Rc::new(Diff::Mod(
                                    (left.clone(), lh),
                                    (right.clone(), rh),
                                ))),
                            ),
                            GraphDiff(FlatDiff::DelL, GDV::Next(y.clone(), (right.clone(), rh))),
                            GraphDiff(FlatDiff::DelR, GDV::Next(x.clone(), (right.clone(), rh))),
                        ]
                    }
                };

                for neighbour in neighbours {
                    let ng = g + cost(&neighbour);

                    if ng < g {
                        parents.insert(neighbour.clone(), gd.clone());

                        let nf = f + hgd(&neighbour);
                        let nmf = max_cost - nf;

                        heap.push((nmf, neighbour));
                    }
                }
            }
            GraphDiff(
                fd,
                GDV::NextLR(((ref l0, l0h), (ref r0, r0h)), ((ref l1, l1h), (ref r1, r1h))),
            ) => {
                let mut th = BinaryHeap::new();
                let max_cost = l0h + r0h;

                th.push((
                    max_cost,
                    GraphDiff(
                        FlatDiff::Start,
                        GDV::Next((l0.clone(), l0h), (r0.clone(), r0h)),
                    ),
                ));
                let dl = diff(max_cost, parents, &mut th);

                th.clear();
                th.push((
                    max_cost,
                    GraphDiff(
                        FlatDiff::Start,
                        GDV::Next((l1.clone(), l1h), (r1.clone(), r1h)),
                    ),
                ));
                let dr = diff(max_cost, parents, &mut th);

                let d = Rc::new(match fd {
                    FlatDiff::TEps(m) => Diff::TEps(m, dl, dr),
                    FlatDiff::TMod(a, b) => Diff::TMod(a, b, dl, dr),
                    _ => unreachable!(),
                });

                return reconstruct_diff(d, gd, parents);
            }
        }
    }

    unreachable!()
}

pub(crate) fn patch<'a>((t, th): TWH<'a>, d: Rc<Diff<'a>>) -> Result<TWH<'a>, PatchError<'a>> {
    match (t.as_ref(), d.as_ref()) {
        (_, Diff::Err(_)) => unreachable!(),
        (_, Diff::Eps) => Ok((t, th)),
        (BCSTree::Leaf(x), Diff::RMod(t, r, br, txt)) => {
            let nx = Data {
                node_type: *t,
                range: r.clone(),
                byte_range: br.clone(),
                text: txt,
                named: x.named,
            };

            Ok((Rc::new(BCSTree::Leaf(nx)), 0))
        }
        (_, Diff::Mod(x, y)) if t == x.0 => Ok(y.clone()),
        (BCSTree::Node(t, x, y), Diff::TEps(td, dx, dy)) if t == td => {
            let (px, pxh) = patch(x.clone(), dx.clone())?;
            let (py, pyh) = patch(y.clone(), dy.clone())?;

            Ok((
                Rc::new(BCSTree::Node(*t, (px, pxh), (py, pyh))),
                pxh.max(pyh),
            ))
        }
        (_, Diff::AddL(td, (x, xh), dy)) => patch((t, th), dy.clone()).map(|(y, yh)| {
            (
                Rc::new(BCSTree::Node(*td, (x.clone(), *xh), (y, yh))),
                (*xh).max(yh),
            )
        }),
        (_, Diff::AddR(td, dx, (y, yh))) => patch((t, th), dx.clone()).map(|(x, xh)| {
            (
                Rc::new(BCSTree::Node(*td, (x, xh), (y.clone(), *yh))),
                (*yh).max(xh),
            )
        }),
        (BCSTree::Node(_, _, y), Diff::DelL(dy)) => patch(y.clone(), dy.clone()),
        (BCSTree::Node(_, x, _), Diff::DelR(dx)) => patch(x.clone(), dx.clone()),
        (BCSTree::Node(t, x, y), Diff::TMod(t0, t1, dx, dy)) if t0 == t => {
            let (px, pxh) = patch(x.clone(), dx.clone())?;
            let (py, pyh) = patch(y.clone(), dy.clone())?;

            Ok((
                Rc::new(BCSTree::Node(*t1, (px, pxh), (py, pyh))),
                pxh.max(pyh),
            ))
        }
        _ => Err(PatchError(t, d)),
    }
}

pub(crate) const LEAF_NIL: BCSTree = BCSTree::Leaf(DATA_NIL);

impl<'a> From<RCSTree<'a>> for (BCSTree<'a>, usize) {
    fn from(t: RCSTree<'a>) -> Self {
        match t {
            RCSTree::Leaf(x) => (BCSTree::Leaf(x), 0),
            RCSTree::Node(m, xs) => {
                if let Some(x) = xs.car() {
                    /* unwrap is safe */
                    match xs.cdr().unwrap().as_ref() {
                        List::Nil => {
                            let (bx, bxh) = x.as_ref().clone().into();

                            (
                                BCSTree::Node(m, (Rc::new(bx), bxh), (Rc::new(LEAF_NIL), 0)),
                                bxh + 1,
                            )
                        }
                        xs => {
                            let (bx, bxh) = x.as_ref().clone().into();
                            let (bxs, bxsh) = RCSTree::Node(META_CONS, xs.clone()).into();

                            (
                                BCSTree::Node(m, (Rc::new(bx), bxh), (Rc::new(bxs), bxsh)),
                                bxh.max(bxsh),
                            )
                        }
                    }
                } else {
                    let nil_rc = (Rc::new(LEAF_NIL), 0);
                    (BCSTree::Node(m, nil_rc.clone(), nil_rc), 0)
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use tree_sitter::Parser;

    use std::rc::Rc;

    use crate::backend::diff::ered;

    use super::{diff_wrapper, patch, BCSTree, RCSTree};

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
        let bcst: (BCSTree, usize) = rcst.into();
        let bcst = (Rc::new(bcst.0), bcst.1);

        let diff = diff_wrapper(bcst.clone(), bcst.clone());

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
        let lbcst: (BCSTree, usize) = lrcst.into();
        let lbcst = (Rc::new(lbcst.0), lbcst.1);

        let rrcst = RCSTree::from(rnode, right);
        let rbcst: (BCSTree, usize) = rrcst.into();
        let rbcst = (Rc::new(rbcst.0), rbcst.1);

        let diff = diff_wrapper(lbcst.clone(), rbcst.clone());

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
        let lbcst: (BCSTree, usize) = lrcst.into();
        let lbcst = (Rc::new(lbcst.0), lbcst.1);

        let rrcst = RCSTree::from(rnode, right);
        let rbcst: (BCSTree, usize) = rrcst.into();
        let rbcst = (Rc::new(rbcst.0), rbcst.1);

        let diff = diff_wrapper(lbcst.clone(), rbcst.clone());

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
        let lbcst: (BCSTree, usize) = lrcst.into();
        let lbcst = (Rc::new(lbcst.0), lbcst.1);

        let rrcst = RCSTree::from(rnode, right);
        let rbcst: (BCSTree, usize) = rrcst.into();
        let rbcst = (Rc::new(rbcst.0), rbcst.1);

        let diff = diff_wrapper(lbcst.clone(), rbcst.clone());

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
        let bcst: (BCSTree, usize) = rcst.into();
        let bcst = Rc::new(bcst.0);

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
        let lbcst: (BCSTree, usize) = lrcst.into();
        let lbcst = (Rc::new(lbcst.0), lbcst.1);

        let rrcst = RCSTree::from(rnode, right);
        let rbcst: (BCSTree, usize) = rrcst.into();
        let rbcst = (Rc::new(rbcst.0), rbcst.1);

        let diff0 = diff_wrapper(lbcst.clone(), rbcst.clone());

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
        let lbcst: (BCSTree, usize) = lrcst.into();
        let lbcst = (Rc::new(lbcst.0), lbcst.1);

        let rrcst = RCSTree::from(rnode, right);
        let rbcst: (BCSTree, usize) = rrcst.into();
        let rbcst = (Rc::new(rbcst.0), rbcst.1);

        let diff0 = diff_wrapper(lbcst.clone(), rbcst.clone());

        let patch0 = patch(lbcst.clone(), diff0.clone()).unwrap();

        let diff1 = ered(diff0);

        let patch1 = patch(lbcst, diff1).unwrap();

        assert_eq!(patch0, patch1)
    }
}
