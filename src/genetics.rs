use rust_monster::ga::ga_random::*;

use ::control::Control;
use ::morphology::Morphology;

pub trait Genetics
{
    fn crossover(&self, other: &Self, random_ctx: &mut GARandomCtx) -> Self;
    fn mutate(&self, random_ctx: &mut GARandomCtx);
}
