//
// LAYERING - THIS IS THE ONLY PLACE ALLOWED TO INCLUDE rust_monster
extern crate rust_monster;
use self::rust_monster::ga::ga_population::*;
// Alias GAIndividual
pub use self::rust_monster::ga::ga_core::GAIndividual as PolyminiGAIndividual;
// Alias GARandomCtx
pub use self::rust_monster::ga::ga_random::GARandomCtx as PolyminiRandomCtx;
// Alias SimpleGA
pub use self::rust_monster::ga::ga_simple::SimpleGeneticAlgorithm as PolyminiGA;
//
//

pub type PolyminiPopulationIter<'a, T> = GAPopulationRawIterator<'a, T>;

pub trait Genetics
{
    fn crossover(&self, other: &Self, random_ctx: &mut PolyminiRandomCtx) -> Self;
    fn mutate(&self, random_ctx: &mut PolyminiRandomCtx);
}

pub struct PolyminiGeneration<T: PolyminiGAIndividual>
{
    individuals: GAPopulation<T>
}
impl<T: PolyminiGAIndividual> PolyminiGeneration<T>
{
    pub fn new(pop: Vec<T>) -> PolyminiGeneration<T>
    {
        PolyminiGeneration { individuals: GAPopulation::new(pop, 
                                                            GAPopulationSortOrder::HighIsBest) }
    }
    pub fn get_individual(&self, i:usize) -> &T
    {
        self.individuals.individual(i, GAPopulationSortBasis::Raw)
    }

    pub fn get_individual_mut(&mut self, i:usize) -> &mut T
    {
        self.individuals.individual_mut(i, GAPopulationSortBasis::Raw)
    }

    pub fn size(&self) -> usize
    {
        self.individuals.size()
    }

    pub fn iter(&self) -> PolyminiPopulationIter<T>
    {
        self.individuals.raw_score_iterator()
    }
}
