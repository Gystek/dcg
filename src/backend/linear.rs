use std::{cmp::Ordering, rc::Rc};

use imara_diff::{Algorithm, Diff, InternedInput};

use crate::backend::ADDR_BYTES;

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LinDiff<'a> {
    Add(&'a str),
    Del,
    Eps,
}

#[derive(Clone, Debug)]
pub(crate) enum PatchError<'a> {
    Empty(LinDiff<'a>),
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub(crate) struct MergeConflict<'a>(pub(crate) LinDiff<'a>, pub(crate) LinDiff<'a>);

pub(crate) fn diff<'a>(
    left: &'a str,
    right: &'a str,
    left_lines: &[&'a str],
    right_lines: &[&'a str],
) -> Vec<LinDiff<'a>> {
    let input = InternedInput::new(left, right);

    let mut diff = Diff::compute(Algorithm::Histogram, &input);
    diff.postprocess_lines(&input);

    let mut i = 0;
    let mut j = 0;

    let mut lin_diff = vec![];

    while i < left_lines.len() || j < right_lines.len() {
        match (
            i < left_lines.len() && diff.is_removed(i as u32),
            j < right_lines.len() && diff.is_added(j as u32),
        ) {
            (true, true) => {
                lin_diff.push(LinDiff::Del);
                lin_diff.push(LinDiff::Add(right_lines[j]));

                i += 1;
                j += 1;
            }
            (false, false) => {
                lin_diff.push(LinDiff::Eps);

                i += 1;
                j += 1;
            }
            (true, false) => {
                lin_diff.push(LinDiff::Del);

                i += 1;
            }
            (false, true) => {
                lin_diff.push(LinDiff::Add(right_lines[j]));

                j += 1;
            }
        }
    }

    lin_diff
}

pub(crate) fn patch<'a>(
    left: &'a [&'a str],
    patch: &[LinDiff<'a>],
) -> Result<Vec<&'a str>, PatchError<'a>> {
    let mut i = 0;
    let mut right = vec![];

    for &d in patch.iter() {
        if let LinDiff::Add(x) = d {
            right.push(x);
        } else {
            if i >= left.len() {
                return Err(PatchError::Empty(d));
            }

            let x = left[i];
            i += 1;

            if let LinDiff::Eps = d {
                right.push(x);
            }
        }
    }

    Ok(right)
}

pub(crate) fn merge<'a>(
    left: &[LinDiff<'a>],
    right: &[LinDiff<'a>],
    conflicts: &mut Vec<MergeConflict<'a>>,
) -> Vec<LinDiff<'a>> {
    let mut i = 0;
    let mut j = 0;

    let mut out = Vec::new();

    while i < left.len() && j < right.len() {
        let ld = left[i];
        let rd = right[j];

        if ld == rd {
            out.push(ld);
            i += 1;
            j += 1;
        } else {
            match (ld, rd) {
                (LinDiff::Add(x), LinDiff::Del) | (LinDiff::Add(x), LinDiff::Eps) => {
                    out.push(LinDiff::Add(x));
                    i += 1;
                }
                (LinDiff::Eps, LinDiff::Add(x)) | (LinDiff::Del, LinDiff::Add(x)) => {
                    out.push(LinDiff::Add(x));
                    j += 1;
                }
                (LinDiff::Del, LinDiff::Eps) | (LinDiff::Eps, LinDiff::Del) => {
                    out.push(LinDiff::Del);
                    i += 1;
                    j += 1;
                }
                _ => {
                    conflicts.push(MergeConflict(ld, rd));
                    i += 1;
                    j += 1;
                }
            }
        }
    }

    if i < left.len() {
        out.extend(&left[i..]);
    }

    if j < right.len() {
        out.extend(&right[j..]);
    }

    out
}

/*
 * multi-byte values are stored in little endian
 * order.  strings are null-terminated.
 */
pub(crate) fn serialise<'a>(dd: &[LinDiff<'a>]) -> Vec<u8> {
    let mut out = vec![];

    out.extend(dd.len().to_le_bytes());

    for d in dd {
        match d {
            LinDiff::Eps => out.push(0),
            LinDiff::Del => out.push(1),
            LinDiff::Add(x) => {
                out.push(2);
                out.extend(x.as_bytes());
                out.push(0);
            }
        }
    }

    out
}

pub(crate) fn deserialise<'a>(b: &'a [u8]) -> Vec<LinDiff<'a>> {
    let mut i = ADDR_BYTES;

    let mut out = Vec::with_capacity(usize::from_le_bytes(b[0..i].try_into().unwrap()));

    while i < b.len() {
        match b[i] {
            0 => out.push(LinDiff::Eps),
            1 => out.push(LinDiff::Del),
            2 => {
                let j = i + 1;

                while i < b.len() && b[i] != 0 {
                    i += 1;
                }

                out.push(LinDiff::Add(unsafe { str::from_utf8_unchecked(&b[j..i]) }));
            }
            _ => unreachable!(),
        }

        i += 1;
    }

    out
}

#[cfg(test)]
mod test {
    use super::{deserialise, diff, merge, patch, serialise};

    #[test]
    fn diff0() {
        let left = r#"
first line is the same
second line is different
third line is new
fourth line is the same"#;

        let right = r#"
first line is the same
second line is not the same
fourth line is the same"#;

        let left_lines = left.lines().collect::<Vec<&str>>();
        let right_lines = right.lines().collect::<Vec<&str>>();

        let diff = diff(left, right, &left_lines, &right_lines);
        let patch = patch(&left_lines, &diff).unwrap();

        assert_eq!(right, patch.join("\n"));
    }

    #[test]
    fn diff1() {
        let left = r#""#;

        let right = r#"left one is empty
diff should be full of `Add`s"#;

        let left_lines = left.lines().collect::<Vec<&str>>();
        let right_lines = right.lines().collect::<Vec<&str>>();

        let diff = diff(left, right, &left_lines, &right_lines);
        let patch = patch(&left_lines, &diff).unwrap();

        assert_eq!(right, patch.join("\n"));
    }

    #[test]
    fn diff2() {
        let left = r#"right one is empty
diff should be full of `Del`s"#;

        let right = r#""#;

        let left_lines = left.lines().collect::<Vec<&str>>();
        let right_lines = right.lines().collect::<Vec<&str>>();

        let diff = diff(left, right, &left_lines, &right_lines);
        let patch = patch(&left_lines, &diff).unwrap();

        assert_eq!(right, patch.join("\n"));
    }

    #[test]
    fn merge0() {
        let base = "line one\nline two\n";
        let left = r#"line one
new second line
line two
"#;
        let right = r#"line one
line two
new third line
"#;

        let base_lines = base.lines().collect::<Vec<&str>>();
        let left_lines = left.lines().collect::<Vec<&str>>();
        let right_lines = right.lines().collect::<Vec<&str>>();

        let diff_bl = diff(base, left, &base_lines, &left_lines);
        let diff_br = diff(base, right, &base_lines, &right_lines);

        println!("bl: {:#?}", diff_bl);
        println!("br: {:#?}", diff_br);

        let mut conflicts = Vec::new();
        let merge = merge(&diff_bl, &diff_br, &mut conflicts);

        println!("m: {:#?}", merge);

        let expected = r#"line one
new second line
line two
new third line"#;

        assert!(conflicts.is_empty());
        assert_eq!(expected, patch(&base_lines, &merge).unwrap().join("\n"));
    }

    #[test]
    fn merge1() {
        let base = "line one\nline two\n";
        let left = r#"line one
contradictory deletion
"#;
        let right = r#"line one
other contradictory deletion
new third line
"#;

        let base_lines = base.lines().collect::<Vec<&str>>();
        let left_lines = left.lines().collect::<Vec<&str>>();
        let right_lines = right.lines().collect::<Vec<&str>>();

        let diff_bl = diff(base, left, &base_lines, &left_lines);
        let diff_br = diff(base, right, &base_lines, &right_lines);

        let mut conflicts = Vec::new();
        merge(&diff_bl, &diff_br, &mut conflicts);

        assert!(!conflicts.is_empty());
    }

    #[test]
    fn merge2() {
        let base = "line one\nline two\n";
        let left = r#"line one
deletion and addition
"#;
        let right = r#"line one
line two
third addition
"#;

        let base_lines = base.lines().collect::<Vec<&str>>();
        let left_lines = left.lines().collect::<Vec<&str>>();
        let right_lines = right.lines().collect::<Vec<&str>>();

        let diff_bl = diff(base, left, &base_lines, &left_lines);
        let diff_br = diff(base, right, &base_lines, &right_lines);

        let mut conflicts = Vec::new();
        let merge = merge(&diff_bl, &diff_br, &mut conflicts);

        println!("{:#?}", merge);

        assert!(!conflicts.is_empty());
    }

    #[test]
    fn merge3() {
        let base = "line one\nline two\n";
        let left = r#"line one
"#;
        let right = r#"line one
test
line two
"#;

        let base_lines = base.lines().collect::<Vec<&str>>();
        let left_lines = left.lines().collect::<Vec<&str>>();
        let right_lines = right.lines().collect::<Vec<&str>>();

        let diff_bl = diff(base, left, &base_lines, &left_lines);
        let diff_br = diff(base, right, &base_lines, &right_lines);

        println!("bl: {:#?}", diff_bl);
        println!("br: {:#?}", diff_br);

        let mut conflicts = Vec::new();
        let merge = merge(&diff_bl, &diff_br, &mut conflicts);

        println!("{:#?}", merge);

        let expected = r#"line one
test"#;

        assert!(conflicts.is_empty());
        assert_eq!(expected, patch(&base_lines, &merge).unwrap().join("\n"));
    }

    #[test]
    fn conservation() {
        let left = r#"the
history of all hitherto existing
societies
has been
the history of
class struggle
"#;

        /* grammar mistake is intentional */
        let right = r#"the
industrial revolution and
its consequences
has been
a disaster for the
human race
"#;

        let left_lines = left.lines().collect::<Vec<&str>>();
        let right_lines = right.lines().collect::<Vec<&str>>();

        let diff = diff(left, right, &left_lines, &right_lines);

        assert_eq!(diff, deserialise(&serialise(&diff)));
    }
}
