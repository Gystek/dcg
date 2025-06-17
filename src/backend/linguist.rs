use glob::Pattern;

use crate::backend::languages::{self, Languages};
use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs::File,
    io::{self, prelude::*, BufReader, Lines},
    path::Path,
};

pub(crate) fn get_ts_language(lng: Languages) -> Option<tree_sitter::Language> {
    match lng {
        Languages::Agda => Some(tree_sitter_agda::LANGUAGE.into()),
        Languages::Bash => Some(tree_sitter_bash::LANGUAGE.into()),
        Languages::CSharp => Some(tree_sitter_c_sharp::LANGUAGE.into()),
        Languages::Cpp => Some(tree_sitter_cpp::LANGUAGE.into()),
        Languages::C => Some(tree_sitter_cpp::LANGUAGE.into()),
        Languages::Css => Some(tree_sitter_css::LANGUAGE.into()),
        Languages::ErbEjs => Some(tree_sitter_embedded_template::LANGUAGE.into()),
        Languages::Go => Some(tree_sitter_go::LANGUAGE.into()),
        Languages::Haskell => Some(tree_sitter_haskell::LANGUAGE.into()),
        Languages::Html => Some(tree_sitter_html::LANGUAGE.into()),
        Languages::Java => Some(tree_sitter_java::LANGUAGE.into()),
        Languages::Javascript => Some(tree_sitter_javascript::LANGUAGE.into()),
        Languages::Json => Some(tree_sitter_json::LANGUAGE.into()),
        Languages::Julia => Some(tree_sitter_julia::LANGUAGE.into()),
        Languages::Ocaml => Some(tree_sitter_ocaml::LANGUAGE_OCAML.into()),
        Languages::Php => Some(tree_sitter_php::LANGUAGE_PHP.into()),
        Languages::Python => Some(tree_sitter_python::LANGUAGE.into()),
        Languages::Regex => Some(tree_sitter_regex::LANGUAGE.into()),
        Languages::Ruby => Some(tree_sitter_ruby::LANGUAGE.into()),
        Languages::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
        Languages::Scala => Some(tree_sitter_rust::LANGUAGE.into()),
        Languages::Typescript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        Languages::Verilog => Some(tree_sitter_verilog::LANGUAGE.into()),
        Languages::PlainText => None,
        Languages::Binary => None,
    }
}

pub(crate) fn guess_language(
    file: &Path,
    filenames: &BTreeMap<Languages, Vec<Pattern>>,
    shebang: &BTreeMap<Languages, Vec<Pattern>>,
    modelines: &BTreeMap<Languages, Vec<Pattern>>,
    heuristics: &BTreeMap<Languages, Vec<Pattern>>,
) -> io::Result<Languages> {
    match guess_filenames(file, filenames) {
        Some(lng) => Ok(lng),
        None => {
            let mut r = BufReader::new(File::open(file)?);
            let mut lines = r.by_ref().lines();

            let n0 = lines.next().map_or(Ok(None), |x| x.map(Some))?;

            if let Some(lng) = guess_shebang(file, shebang, &n0) {
                Ok(lng)
            } else if let Some(lng) = guess_modelines(file, modelines, lines, n0)? {
                Ok(lng)
            } else {
                r.rewind()?;

                if let Some(lng) = guess_heuristics(file, heuristics, r.by_ref().lines())? {
                    Ok(lng)
                } else {
                    r.rewind()?;
                    plain_or_binary(file, r)
                }
            }
        }
    }
}

fn guess_filenames(
    file: &Path,
    filenames: &BTreeMap<Languages, Vec<Pattern>>,
) -> Option<Languages> {
    let fname = file.file_name().and_then(OsStr::to_str).unwrap_or("");

    for (lang, patterns) in filenames {
        for pattern in patterns {
            if pattern.matches(fname) {
                return Some(*lang);
            }
        }
    }

    None
}

fn guess_shebang(
    file: &Path,
    shebang: &BTreeMap<Languages, Vec<Pattern>>,
    first: &Option<String>,
) -> Option<Languages> {
    let first = first.as_deref().unwrap_or("");

    for (lang, patterns) in shebang {
        for pattern in patterns {
            if pattern.matches(first) {
                return Some(*lang);
            }
        }
    }

    None
}

const MODELINE_LINE_COUNT: usize = 5;

fn guess_modelines(
    file: &Path,
    modelines: &BTreeMap<Languages, Vec<Pattern>>,
    lines: Lines<&mut BufReader<File>>,
    first: Option<String>,
) -> io::Result<Option<Languages>> {
    let first = first.as_deref().unwrap_or("");
    /* we look at the MODELINE_LINE_COUNT first and last lines
     * of the file
     */
    let mut last = Vec::with_capacity(MODELINE_LINE_COUNT);

    last.push(first.to_string());

    for (lang, patterns) in modelines {
        for pattern in patterns {
            if pattern.matches(first) {
                return Ok(Some(*lang));
            }
        }
    }

    for (i, line) in lines.enumerate() {
        let line = line?;

        if i + 1 < MODELINE_LINE_COUNT {
            for (lang, patterns) in modelines {
                for pattern in patterns {
                    if pattern.matches(&line) {
                        return Ok(Some(*lang));
                    }
                }
            }
        }

        if last.len() >= MODELINE_LINE_COUNT {
            last.remove(0);
            last.push(line);
        }
    }

    for line in last {
        for (lang, patterns) in modelines {
            for pattern in patterns {
                if pattern.matches(&line) {
                    return Ok(Some(*lang));
                }
            }
        }
    }

    Ok(None)
}

fn guess_heuristics(
    file: &Path,
    heuristics: &BTreeMap<Languages, Vec<Pattern>>,
    lines: Lines<&mut BufReader<File>>,
) -> io::Result<Option<Languages>> {
    for line in lines {
        let line = line?;

        for (lang, patterns) in heuristics {
            for pattern in patterns {
                if pattern.matches(&line) {
                    return Ok(Some(*lang));
                }
            }
        }
    }

    Ok(None)
}

fn plain_or_binary(file: &Path, reader: BufReader<File>) -> io::Result<Languages> {
    Ok(Languages::PlainText)
}
