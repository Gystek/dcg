use crate::backend::{bcst::BCSTree, diff::Diff};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub(crate) struct PatchError(pub(crate) Rc<BCSTree>, pub(crate) Rc<Diff>);
