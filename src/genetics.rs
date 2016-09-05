use rust_monster::ga::ga_random::*;
use rust_monster::ga::ga_population::*;

use ::polymini::*;

pub type PolyminiRandomCtx = GARandomCtx;

pub struct Splice {} 
pub trait Genetics
{
    fn crossover(&self, other: &Self, random_ctx: &mut PolyminiRandomCtx) -> Self;
    fn mutate(&self, random_ctx: &mut PolyminiRandomCtx);
}

pub struct PolyminiGeneration
{
    pub individuals: GAPopulation<Polymini>
}
impl PolyminiGeneration
{

}
