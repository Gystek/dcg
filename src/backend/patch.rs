//! Errors arising when a file is patched
use crate::backend::{bcst::BCSTree, diff::Diff};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub(crate) struct PatchError<'a>(pub(crate) Rc<BCSTree<'a>>, pub(crate) Rc<Diff<'a>>);
