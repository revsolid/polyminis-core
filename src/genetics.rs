//
// LAYERING - THIS IS THE ONLY PLACE ALLOWED TO INCLUDE rust_monster
extern crate rust_monster;
use self::rust_monster::ga::ga_population::*;
// Alias GAIndividual
pub use self::rust_monster::ga::ga_core::GAIndividual as PolyminiGAIndividual;
// Alias GA
pub use self::rust_monster::ga::ga_core::GeneticAlgorithm as PolyminiGA;
// Alias GARandomCtx
pub use self::rust_monster::ga::ga_random::GARandomCtx as PolyminiRandomCtx;
//
//

pub type PolyminiPopulationIter<'a, T> = GAPopulationRawIterator<'a, T>;

pub trait Genetics
{
    fn crossover(&self, other: &Self, random_ctx: &mut PolyminiRandomCtx) -> Self;
    fn mutate(&mut self, random_ctx: &mut PolyminiRandomCtx);
}

pub struct PolyminiGeneration<T: PolyminiGAIndividual>
{
    individuals: GAPopulation<T>
}
impl<T: PolyminiGAIndividual> PolyminiGeneration<T>
{
    pub fn new(pop: Vec<T>) -> PolyminiGeneration<T>
    {
        let mut ga_pop = GAPopulation::new(pop, GAPopulationSortOrder::HighIsBest);
        ga_pop.sort();
        PolyminiGeneration { individuals: ga_pop }
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

pub struct PolyminiGeneticAlgorithm<T: PolyminiGAIndividual>
{
   population: PolyminiGeneration<T> 
}
impl<T: PolyminiGAIndividual> PolyminiGeneticAlgorithm<T>
{
    pub fn new(pop: Vec<T>) -> PolyminiGeneticAlgorithm<T>
    {
        PolyminiGeneticAlgorithm { population: PolyminiGeneration::new(pop) }
    }
    
    pub fn get_population(&self) -> &PolyminiGeneration<T>
    {
        &self.population
    }

    pub fn get_population_mut(&mut self) -> &mut PolyminiGeneration<T>
    {
        &mut self.population
    }
}

impl<T: PolyminiGAIndividual> PolyminiGA<T> for PolyminiGeneticAlgorithm<T>
{
    fn population(&mut self) -> &mut GAPopulation<T>
    {
        &mut self.population.individuals
    }
}
