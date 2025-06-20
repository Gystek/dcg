use std::{cmp::Ordering, rc::Rc};

use imara_diff::{Algorithm, Diff, InternedInput};

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
    vec![]
}

#[cfg(test)]
mod test {
    use super::{diff, patch};

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
}
