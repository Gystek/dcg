use crate::backend::languages::{self, Languages};
use std::{collections::BTreeMap, path::Path};

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

pub(crate) fn guess_language(file: &Path) -> Languages {
    guess_filenames(file)
        .or_else(|| guess_shebangs(file))
        .or_else(|| guess_modelines(file))
        .or_else(|| guess_heuristics(file))
        .unwrap_or_else(|| plain_or_binary(file))
}

fn guess_filenames(file: &Path) -> Option<Languages> {
    None
}

fn guess_shebangs(file: &Path) -> Option<Languages> {
    None
}

fn guess_modelines(file: &Path) -> Option<Languages> {
    None
}

fn guess_heuristics(file: &Path) -> Option<Languages> {
    None
}

fn plain_or_binary(file: &Path) -> Languages {
    Languages::PlainText
}
