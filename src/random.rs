//
// LAYERING - THIS IS THE ONLY (other) PLACE ALLOWED TO INCLUDE rust_monster (besides genetics)
// TODO: This should be separated in a different Crate
// Alias GARandomCtx
extern crate rust_monster;
pub use self::rust_monster::ga::ga_random::GARandomCtx as PolyminiRandomCtx;
