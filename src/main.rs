use std::{env, fs::File, io::Read, path::Path, process::exit, rc::Rc};

use backend::{
    bcst::{diff_wrapper, BCSTree},
    diff::ered,
    languages::{
        compile_filenames_map, compile_heuristics_map, compile_modelines_map, compile_shebang_map,
        init_all_maps,
    },
    linguist::{get_ts_language, guess_language},
    rcst::RCSTree,
};
use tree_sitter::Parser;

mod backend;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        exit(1);
    }

    let left = &args[1];
    let right = &args[2];

    /* language identification */
    init_all_maps();
    let filenames = compile_filenames_map();
    let shebang = compile_shebang_map();
    let modelines = compile_modelines_map();
    let heuristics = compile_heuristics_map();

    let left_lang = guess_language(
        Path::new(left),
        &filenames,
        &shebang,
        &modelines,
        &heuristics,
    )
    .unwrap();
    let right_lang = guess_language(
        Path::new(right),
        &filenames,
        &shebang,
        &modelines,
        &heuristics,
    )
    .unwrap();

    if left_lang != right_lang {
        eprintln!("Cannot AST-diff two files not of the same language:");
        eprintln!(
            "\t'{}' is in {:?} whereas '{}' is in {:?}",
            left, left_lang, right, right_lang
        );
        return;
    }

    println!("Identified language: {:?}", left_lang);

    let ts_language = get_ts_language(left_lang).unwrap();

    /* file reading */
    let mut lf = File::open(left).unwrap();
    let mut rf = File::open(right).unwrap();

    let mut lc = String::new();
    let mut rc = String::new();

    lf.read_to_string(&mut lc).unwrap();
    rf.read_to_string(&mut rc).unwrap();

    /* parsing */

    let mut parser = Parser::new();

    parser.set_language(&ts_language).unwrap();

    let ltree = parser.parse(&lc, None).unwrap();
    let rtree = parser.parse(&rc, None).unwrap();

    let lnode = ltree.root_node();
    let rnode = rtree.root_node();

    let lrcst = RCSTree::from(lnode, &lc);
    let rrcst = RCSTree::from(rnode, &rc);

    let lbcst: (BCSTree, usize) = lrcst.into();
    let rbcst: (BCSTree, usize) = rrcst.into();
    let lbcst = (Rc::new(lbcst.0), lbcst.1);
    let rbcst = (Rc::new(rbcst.0), lbcst.1);

    /* diffing */

    let diff = ered(diff_wrapper(lbcst.clone(), rbcst.clone()));

    println!("{:#?}", diff);
    println!("weight {}", diff.weight());
}
