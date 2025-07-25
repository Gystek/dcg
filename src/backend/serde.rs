//! [de]serialise `Diff`s to binary
use crate::backend::{
    bcst::{BCSTree, Twh},
    data::Data,
    diff::Diff,
    metadata::Metadata,
    ADDR_BYTES,
};

use std::{collections::HashMap, ops::Range, rc::Rc};

pub(crate) type Ranges = HashMap<(Range<(usize, usize)>, Range<usize>), usize>;
pub(crate) type TextRanges<'a> = HashMap<(Range<(usize, usize)>, Range<usize>, &'a str), usize>;

type VecRanges = Vec<(Range<(usize, usize)>, Range<usize>)>;
type VecTextRanges<'a> = Vec<(Range<(usize, usize)>, Range<usize>, &'a str)>;

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
struct SerData {
    node_type: u16,
    text: bool,
    range: usize,
    named: bool,
}

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
enum SerBCSTree {
    Leaf(SerData),
    Node(Metadata, (Rc<SerBCSTree>, usize), (Rc<SerBCSTree>, usize)),
}

fn process_ranges((t, _): Twh<'_>, r: &mut Ranges) {
    match t.as_ref() {
        BCSTree::Leaf(d) => {
            if r.get(&(d.range.clone(), d.byte_range.clone())).is_none() {
                r.insert((d.range.clone(), d.byte_range.clone()), r.len());
            }
        }
        BCSTree::Node(_, left, right) => {
            process_ranges(left.clone(), r);
            process_ranges(right.clone(), r);
        }
    }
}

fn process_text_ranges<'a>((t, _): Twh<'a>, r: &mut TextRanges<'a>) {
    match t.as_ref() {
        BCSTree::Leaf(d) => {
            if r.get(&(d.range.clone(), d.byte_range.clone(), d.text))
                .is_none()
            {
                r.insert((d.range.clone(), d.byte_range.clone(), d.text), r.len());
            }
        }
        BCSTree::Node(_, left, right) => {
            process_text_ranges(left.clone(), r);
            process_text_ranges(right.clone(), r);
        }
    }
}

/*
 * practically limited to 65535 node types
 */
fn serialise_tree<'a>(t: Twh<'a>, text: bool, r: &mut Ranges, tr: &mut TextRanges<'a>) -> Vec<u8> {
    if text {
        process_text_ranges(t.clone(), tr);
    } else {
        process_ranges(t.clone(), r);
    }

    let (st, sth) = tree_to_sertree(t, text, r, tr);

    serialise_sertree(Rc::new(st), sth)
}

fn tree_to_sertree<'a>(
    (t, th): Twh<'a>,
    text: bool,
    r: &Ranges,
    tr: &TextRanges<'a>,
) -> (SerBCSTree, usize) {
    match t.as_ref() {
        BCSTree::Leaf(d) => (
            SerBCSTree::Leaf(SerData {
                node_type: d.node_type.map(|x| x + 1).unwrap_or(0),
                text,
                range: *if text {
                    tr.get(&(d.range.clone(), d.byte_range.clone(), d.text))
                } else {
                    r.get(&(d.range.clone(), d.byte_range.clone()))
                }
                .unwrap(),
                named: d.named,
            }),
            th,
        ),
        BCSTree::Node(m, left, right) => {
            let (sl, slh) = tree_to_sertree(left.clone(), text, r, tr);
            let (sr, srh) = tree_to_sertree(right.clone(), text, r, tr);
            (
                SerBCSTree::Node(*m, (Rc::new(sl), slh), (Rc::new(sr), srh)),
                th,
            )
        }
    }
}

fn serialise_sertree(t: Rc<SerBCSTree>, th: usize) -> Vec<u8> {
    match t.as_ref() {
        SerBCSTree::Leaf(d) => {
            let mut v = vec![0];

            v.extend_from_slice(&d.node_type.to_le_bytes());
            v.push(d.text as u8);
            v.extend_from_slice(&d.range.to_le_bytes());
            v.push(d.named as u8);

            v
        }
        SerBCSTree::Node(m, (left, lth), (right, rth)) => {
            let mut v = vec![1];

            v.extend_from_slice(&th.to_le_bytes());
            v.extend_from_slice(&m.node_type.map(|x| x + 1).unwrap_or(0).to_le_bytes());
            v.extend_from_slice(&serialise_sertree(left.clone(), *lth));
            v.extend_from_slice(&serialise_sertree(right.clone(), *rth));

            v
        }
    }
}

/*
 * practically limited to 65535 node types
 *
 * multi-byte values are stored in little endian
 * order.  strings are null-terminated.
 */
pub(crate) fn serialise<'a>(d: Rc<Diff<'a>>, r: &mut Ranges, tr: &mut TextRanges<'a>) -> Vec<u8> {
    match d.as_ref() {
        Diff::Eps => vec![0],
        Diff::Err(_) => unreachable!(),
        Diff::RMod(nt, nr, br, ns) => {
            let mut v = vec![1];

            v.extend_from_slice(&(*nt).map(|x| x + 1).unwrap_or(0).to_le_bytes());
            v.extend_from_slice(
                &if let Some(x) = tr.get(&(nr.clone(), br.clone(), ns)) {
                    *x
                } else {
                    tr.insert((nr.clone(), br.clone(), ns), tr.len());
                    tr.len() - 1
                }
                .to_le_bytes(),
            );

            v
        }
        Diff::TEps(m, left, right) => {
            let mut v = vec![2];

            v.extend_from_slice(&m.node_type.map(|x| x + 1).unwrap_or(0).to_le_bytes());
            v.extend_from_slice(&serialise(left.clone(), r, tr));
            v.extend_from_slice(&serialise(right.clone(), r, tr));

            v
        }
        Diff::Mod(from, to) => {
            let mut v = vec![3];

            v.extend_from_slice(&serialise_tree(from.clone(), false, r, tr));
            v.extend_from_slice(&serialise_tree(to.clone(), true, r, tr));

            v
        }
        Diff::TMod(f, t, left, right) => {
            let mut v = vec![4];

            v.extend_from_slice(&f.node_type.map(|x| x + 1).unwrap_or(0).to_le_bytes());
            v.extend_from_slice(&t.node_type.map(|x| x + 1).unwrap_or(0).to_le_bytes());
            v.extend_from_slice(&serialise(left.clone(), r, tr));
            v.extend_from_slice(&serialise(right.clone(), r, tr));

            v
        }
        Diff::AddL(m, t, d) => {
            let mut v = vec![5];

            v.extend_from_slice(&m.node_type.map(|x| x + 1).unwrap_or(0).to_le_bytes());
            v.extend_from_slice(&serialise_tree(t.clone(), true, r, tr));
            v.extend_from_slice(&serialise(d.clone(), r, tr));

            v
        }
        Diff::AddR(m, d, t) => {
            let mut v = vec![6];

            v.extend_from_slice(&m.node_type.map(|x| x + 1).unwrap_or(0).to_le_bytes());
            v.extend_from_slice(&serialise_tree(t.clone(), true, r, tr));
            v.extend_from_slice(&serialise(d.clone(), r, tr));

            v
        }
        Diff::DelL(d) => {
            let mut v = vec![7];

            v.extend_from_slice(&serialise(d.clone(), r, tr));

            v
        }
        Diff::DelR(d) => {
            let mut v = vec![8];

            v.extend_from_slice(&serialise(d.clone(), r, tr));

            v
        }
    }
}

fn u16_to_nt(x: u16) -> Option<u16> {
    if x == 0 {
        None
    } else {
        Some(x - 1)
    }
}

fn deserialise_tree<'a>(
    b: &'a [u8],
    t: &'a str,
    r: &VecRanges,
    tr: &VecTextRanges<'a>,
) -> (Twh<'a>, &'a [u8]) {
    match b[0] {
        0 => {
            let nt = u16_to_nt(u16::from_le_bytes(b[1..3].try_into().unwrap()));
            let text = b[3] != 0;
            let rn = usize::from_le_bytes(b[4..4 + ADDR_BYTES].try_into().unwrap());
            let named = b[4 + ADDR_BYTES] != 0;

            (
                (
                    Rc::new(BCSTree::Leaf(Data {
                        node_type: nt,
                        range: if text {
                            tr[rn].0.clone()
                        } else {
                            r[rn].0.clone()
                        },
                        byte_range: if text {
                            tr[rn].1.clone()
                        } else {
                            r[rn].1.clone()
                        },
                        text: if text { tr[rn].2 } else { &t[r[rn].1.clone()] },
                        named,
                    })),
                    0,
                ),
                &b[4 + ADDR_BYTES + 1..],
            )
        }
        1 => {
            let h = usize::from_le_bytes(b[1..1 + ADDR_BYTES].try_into().unwrap());
            let m = u16_to_nt(u16::from_le_bytes(
                b[1 + ADDR_BYTES..1 + ADDR_BYTES + 2].try_into().unwrap(),
            ));

            let (left, b) = deserialise_tree(&b[1 + ADDR_BYTES + 2..], t, r, tr);
            let (right, b) = deserialise_tree(b, t, r, tr);

            (
                (
                    Rc::new(BCSTree::Node(Metadata { node_type: m }, left, right)),
                    h,
                ),
                b,
            )
        }
        _ => unreachable!("badly serialised tree"),
    }
}

pub(crate) fn deserialise<'a>(
    b: &'a [u8],
    t: &'a str,
    r: &VecRanges,
    tr: &VecTextRanges<'a>,
) -> (Diff<'a>, &'a [u8]) {
    match b[0] {
        0 => (Diff::Eps, &b[1..]),
        1 => {
            let t = u16::from_le_bytes(b[1..3].try_into().unwrap());
            let ri = usize::from_le_bytes(b[3..3 + ADDR_BYTES].try_into().unwrap());
            let (r, br, s) = tr.get(ri).unwrap();

            (
                Diff::RMod(u16_to_nt(t), r.clone(), br.clone(), s),
                &b[3 + ADDR_BYTES..],
            )
        }
        2 => {
            let m = u16::from_le_bytes(b[1..3].try_into().unwrap());
            let (left, b) = deserialise(&b[3..], t, r, tr);
            let (right, b) = deserialise(b, t, r, tr);

            (
                Diff::TEps(
                    Metadata {
                        node_type: u16_to_nt(m),
                    },
                    Rc::new(left),
                    Rc::new(right),
                ),
                b,
            )
        }
        3 => {
            let (from, b) = deserialise_tree(&b[1..], t, r, tr);
            let (to, b) = deserialise_tree(b, t, r, tr);

            (Diff::Mod(from, to), b)
        }
        4 => {
            let from = u16::from_le_bytes(b[1..3].try_into().unwrap());
            let to = u16::from_le_bytes(b[3..5].try_into().unwrap());

            let (left, b) = deserialise(&b[5..], t, r, tr);
            let (right, b) = deserialise(b, t, r, tr);

            (
                Diff::TMod(
                    Metadata {
                        node_type: u16_to_nt(from),
                    },
                    Metadata {
                        node_type: u16_to_nt(to),
                    },
                    Rc::new(left),
                    Rc::new(right),
                ),
                b,
            )
        }
        5 => {
            let m = u16::from_le_bytes(b[1..3].try_into().unwrap());
            let (tree, b) = deserialise_tree(&b[3..], t, r, tr);
            let (d, b) = deserialise(b, t, r, tr);

            (
                Diff::AddL(
                    Metadata {
                        node_type: u16_to_nt(m),
                    },
                    tree,
                    Rc::new(d),
                ),
                b,
            )
        }
        6 => {
            let m = u16::from_le_bytes(b[1..3].try_into().unwrap());
            let (tree, b) = deserialise_tree(&b[3..], t, r, tr);
            let (d, b) = deserialise(b, t, r, tr);

            (
                Diff::AddR(
                    Metadata {
                        node_type: u16_to_nt(m),
                    },
                    Rc::new(d),
                    tree,
                ),
                b,
            )
        }
        7 => {
            let (d, b) = deserialise(&b[1..], t, r, tr);

            (Diff::DelL(Rc::new(d)), b)
        }
        8 => {
            let (d, b) = deserialise(&b[1..], t, r, tr);

            (Diff::DelR(Rc::new(d)), b)
        }
        _ => unreachable!("badly serialised diff"),
    }
}

#[cfg(test)]
mod test {
    use tree_sitter::Parser;

    use crate::backend::{
        bcst::{diff_wrapper, BCSTree},
        rcst::RCSTree,
    };
    use std::rc::Rc;

    use super::{deserialise, serialise, Ranges, TextRanges};

    #[test]
    fn conservation() {
        let left = "pub fn foo() {\n  1\n}";
        let right = "fn bar() {\n  baz(5); 'c'\n}";

        let mut parser = Parser::new();

        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();

        let ltree = parser.parse(left, None).unwrap();
        let lnode = ltree.root_node();

        let rtree = parser.parse(right, None).unwrap();
        let rnode = rtree.root_node();

        let rrcst = RCSTree::from(rnode, right);
        let rbcst: (BCSTree, usize) = rrcst.into();
        let rbcst = (Rc::new(rbcst.0), rbcst.1);

        let lrcst = RCSTree::from(lnode, left);
        let lbcst: (BCSTree, usize) = lrcst.into();
        let lbcst = (Rc::new(lbcst.0), lbcst.1);

        let diff = diff_wrapper(lbcst.clone(), rbcst.clone());

        let mut ranges = Ranges::new();
        let mut tranges = TextRanges::new();

        let ser = serialise(diff.clone(), &mut ranges, &mut tranges);

        let mut vr = vec![((0, 0)..(0, 0), 0..0); ranges.len()];
        let mut vtr = vec![((0, 0)..(0, 0), 0..0, ""); tranges.len()];

        for (k, v) in ranges {
            vr[v] = k;
        }

        for (k, v) in tranges {
            vtr[v] = k;
        }

        let (de, _) = deserialise(&ser, left, &vr, &vtr);

        assert_eq!(diff.as_ref(), &de)
    }
}
