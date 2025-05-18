use std::{env, fs::File, io::Read, process::exit, rc::Rc};

use backend::{
    bcst::{diff_wrapper, BCSTree},
    diff::ered,
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

    let mut lf = File::open(left).unwrap();
    let mut rf = File::open(right).unwrap();

    let mut lc = String::new();
    let mut rc = String::new();

    lf.read_to_string(&mut lc).unwrap();
    rf.read_to_string(&mut rc).unwrap();

    let mut parser = Parser::new();

    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .unwrap();

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

    let diff = ered(diff_wrapper(lbcst.clone(), rbcst.clone()));

    println!("{:#?}", diff);
    println!("weight {}", diff.weight());
}
