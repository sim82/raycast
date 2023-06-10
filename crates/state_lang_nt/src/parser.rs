use lrlex::lrlex_mod;
use lrpar::lrpar_mod;

lrlex_mod!("state_bc.l");
lrpar_mod!("state_bc.y");

pub use state_bc_l::lexerdef;
pub use state_bc_y::{parse, token_epp, StateElement, Toplevel, TypedInt, Word};
