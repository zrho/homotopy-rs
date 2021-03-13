pub mod attach;
pub mod common;
pub mod complex;
pub mod contraction;
pub mod diagram;
pub mod expansion;
pub mod factorization;
pub mod hashcons {
    pub use crate::common::hashcons_impl_Generator::*;
    pub use crate::common::hashcons_type_Generator::*;
    pub use crate::diagram::hashcons_impl_Diagram::*;
    pub use crate::diagram::hashcons_impl_DiagramN::*;
    pub use crate::diagram::hashcons_type_Diagram::*;
    pub use crate::diagram::hashcons_type_DiagramN::*;
    pub(crate) use crate::rewrite::hashcons_impl_Cone::*;
    pub use crate::rewrite::hashcons_impl_Rewrite::*;
    pub use crate::rewrite::hashcons_impl_Rewrite0::*;
    pub use crate::rewrite::hashcons_impl_RewriteN::*;
    pub(crate) use crate::rewrite::hashcons_type_Cone::*;
    pub use crate::rewrite::hashcons_type_Rewrite::*;
    pub use crate::rewrite::hashcons_type_Rewrite0::*;
    pub use crate::rewrite::hashcons_type_RewriteN::*;
}
pub mod normalization;
pub mod projection;
pub mod rewrite;
pub mod serialize;
pub mod typecheck;

pub use common::{Boundary, Direction, Generator, Height, SliceIndex};
pub use contraction::Bias;
pub use diagram::{Diagram, DiagramN};
pub use rewrite::{Cospan, Rewrite, Rewrite0, RewriteN};
