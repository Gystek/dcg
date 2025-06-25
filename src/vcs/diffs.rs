use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    rc::Rc,
};

use crate::backend::{
    bcst::{diff_wrapper, BCSTree},
    diff::ered,
    languages::Languages,
    linear::{self},
    linguist::{get_ts_language, guess_language, LinguistState},
    rcst::RCSTree,
    serde::{serialise, Ranges, TextRanges},
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

                let mut ranges = Ranges::new();
                let mut tranges = TextRanges::new();

                Ok(serialise(diff, &mut ranges, &mut tranges))
            } else {
                do_diff_linear(s1, s2)
            }
        } else {
            unreachable!()
        }
    }
}
