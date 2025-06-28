use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    rc::Rc,
};

use crate::backend::{
    bcst::{diff_wrapper, BCSTree},
    diff::{ered, Diff},
    languages::Languages,
    linear,
    linguist::{get_ts_language, guess_language, LinguistState},
    rcst::RCSTree,
    serde::{deserialise, serialise, Ranges, TextRanges},
    ADDR_BYTES,
};

use anyhow::Result;
use flate2::write::GzDecoder;
use tree_sitter::Parser;

#[derive(Debug, Copy, Clone)]
pub(crate) enum DiffType {
    Linear(Languages, Languages),
    Tree(Languages),
    Binary,
    FromBinary(Languages),
}

impl DiffType {
    pub(crate) fn serialise(self) -> Vec<u8> {
        match self {
            Self::Binary => vec![0],
            Self::FromBinary(l) => {
                let mut v = vec![1];
                v.push(l as u8);
                v
            }
            Self::Tree(l) => {
                let mut v = vec![2];
                v.push(l as u8);
                v
            }
            Self::Linear(l0, l1) => {
                let mut v = vec![3];
                v.push(l0 as u8);
                v.push(l1 as u8);
                v
            }
        }
    }
    pub(crate) fn deserialise(v: &[u8]) -> Self {
        match v[0] {
            0 => Self::Binary,
            1 => Self::FromBinary(Languages::try_from(v[1]).unwrap()),
            2 => Self::Tree(Languages::try_from(v[1]).unwrap()),
            3 => Self::Linear(
                Languages::try_from(v[1]).unwrap(),
                Languages::try_from(v[2]).unwrap(),
            ),
            _ => unreachable!(),
        }
    }
}

pub(crate) fn get_diff_type<P: AsRef<Path>>(
    linguist: LinguistState,
    file1: P,
    file2: P,
) -> Result<DiffType> {
    let lang1 = guess_language(file1.as_ref(), linguist)?;
    let lang2 = guess_language(file2.as_ref(), linguist)?;

    Ok(match (lang1, lang2) {
        /* Binary and FromBinary means deletion + addition */
        (_, Languages::Binary) => DiffType::Binary,
        (Languages::Binary, _) => DiffType::FromBinary(lang2),
        (x, y) if x == y && x != Languages::PlainText => DiffType::Tree(lang1),
        _ => DiffType::Linear(lang1, lang2),
    })
}

fn read_usize(i: &mut usize, v: &[u8]) -> usize {
    let x = usize::from_le_bytes(v[*i..*i + ADDR_BYTES].try_into().unwrap());
    *i += ADDR_BYTES;

    x
}

/// Deserialise the diff from its serialised form alone.
pub(crate) fn deserialise_everything<'a>(v: &'a [u8], left: &'a str) -> Result<Diff<'a>> {
    let mut i = 0;
    let rl = read_usize(&mut i, v);

    let mut vr = vec![((0, 0)..(0, 0), 0..0); rl];

    for _ in 0..rl {
        let r1s0 = read_usize(&mut i, v);
        let r1s1 = read_usize(&mut i, v);
        let r1e0 = read_usize(&mut i, v);
        let r1e1 = read_usize(&mut i, v);

        let r2s = read_usize(&mut i, v);
        let r2e = read_usize(&mut i, v);

        let x = read_usize(&mut i, v);

        vr[x] = ((r1s0, r1s1)..(r1e0, r1e1), r2s..r2e);
    }

    let trl = read_usize(&mut i, v);
    let mut vtr = vec![((0, 0)..(0, 0), 0..0, ""); trl];

    for _ in 0..trl {
        let r1s0 = read_usize(&mut i, v);
        let r1s1 = read_usize(&mut i, v);
        let r1e0 = read_usize(&mut i, v);
        let r1e1 = read_usize(&mut i, v);

        let r2s = read_usize(&mut i, v);
        let r2e = read_usize(&mut i, v);

        let sl = read_usize(&mut i, v);
        let s = str::from_utf8(&v[i..i + sl])?;
        i += sl;

        let x = read_usize(&mut i, v);

        vtr[x] = ((r1s0, r1s1)..(r1e0, r1e1), r2s..r2e, s);
    }

    Ok(deserialise(&v[i..], left, &vr, &vtr).0)
}

/// Serialise the diff along with the ranges.  All numbers are
/// written in little endian.  Strings are not null terminated,
/// as their length is stored.
pub(crate) fn serialise_everything(d: Rc<Diff>) -> Vec<u8> {
    let mut ranges = Ranges::new();
    let mut tranges = TextRanges::new();

    let ds = serialise(d, &mut ranges, &mut tranges);

    let mut ser = Vec::new();

    ser.extend(ranges.len().to_le_bytes());
    for ((range1, range2), idx) in ranges {
        ser.extend(range1.start.0.to_le_bytes());
        ser.extend(range1.start.1.to_le_bytes());
        ser.extend(range1.end.0.to_le_bytes());
        ser.extend(range1.end.1.to_le_bytes());

        ser.extend(range2.start.to_le_bytes());
        ser.extend(range2.end.to_le_bytes());

        ser.extend(idx.to_le_bytes());
    }

    ser.extend(tranges.len().to_le_bytes());
    for ((range1, range2, s), idx) in tranges {
        ser.extend(range1.start.0.to_le_bytes());
        ser.extend(range1.start.1.to_le_bytes());
        ser.extend(range1.end.0.to_le_bytes());
        ser.extend(range1.end.1.to_le_bytes());

        ser.extend(range2.start.to_le_bytes());
        ser.extend(range2.end.to_le_bytes());

        ser.extend(s.len().to_le_bytes());
        ser.extend(s.as_bytes());

        ser.extend(idx.to_le_bytes());
    }

    ser.extend(ds);

    ser
}

fn do_diff_linear(mut s1: String, mut s2: String) -> Result<Vec<u8>> {
    // linear diff requires full lines, as the crate we use for this
    // purpose does not process trimmed lines. thus, to hhandle line
    // displacement in the best way, we have to manually add a line break
    // at the end of each string.
    s1.push('\n');
    s2.push('\n');

    let l1 = s1.lines().collect::<Vec<&str>>();
    let l2 = s2.lines().collect::<Vec<&str>>();

    let diff = linear::diff(&s1, &s2, &l1, &l2);

    Ok(linear::serialise(&diff))
}

pub(crate) fn do_diff<P: AsRef<Path>>(
    difft: DiffType,
    file1: P,
    file2: P,
    encoded: bool,
) -> Result<Vec<u8>> {
    if matches!(difft, DiffType::Binary | DiffType::FromBinary(_)) {
        let mut contents = Vec::new();

        File::open(file2)?.read_to_end(&mut contents)?;

        Ok(contents)
    } else {
        let mut f1 = File::open(file1)?;
        let mut f2 = File::open(file2)?;

        let mut s1 = String::new();
        let mut s2 = String::new();

        if encoded {
            let mut b1 = Vec::new();
            let mut b2 = Vec::new();

            f1.read_to_end(&mut b1)?;
            f2.read_to_end(&mut b2)?;

            let mut writer = Vec::new();
            let mut decoder = GzDecoder::new(writer);

            decoder.write_all(&b1)?;
            writer = decoder.finish()?;

            s1 = String::from_utf8(writer)?;

            let mut writer = Vec::new();
            let mut decoder = GzDecoder::new(writer);

            decoder.write_all(&b2)?;
            writer = decoder.finish()?;

            s2 = String::from_utf8(writer)?;
        } else {
            f1.read_to_string(&mut s1)?;
            f2.read_to_string(&mut s2)?;
        }

        if let DiffType::Linear(_, _) = difft {
            do_diff_linear(s1, s2)
        } else if let DiffType::Tree(lang) = difft {
            let ts_language = get_ts_language(lang).unwrap();
            let mut parser = Parser::new();

            parser.set_language(&ts_language)?;

            if let (Some(t1), Some(t2)) = (parser.parse(&s1, None), parser.parse(&s2, None)) {
                let n1 = t1.root_node();
                let n2 = t2.root_node();

                let r1 = RCSTree::from(n1, &s1);
                let r2 = RCSTree::from(n2, &s2);

                let (b1, bn1): (BCSTree, usize) = r1.into();
                let (b2, bn2): (BCSTree, usize) = r2.into();

                let diff = ered(diff_wrapper((Rc::new(b1), bn1), (Rc::new(b2), bn2)));

                Ok(serialise_everything(diff))
            } else {
                do_diff_linear(s1, s2)
            }
        } else {
            unreachable!()
        }
    }
}
