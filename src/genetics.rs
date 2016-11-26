//
// LAYERING - THIS IS THE ONLY PLACE ALLOWED TO INCLUDE rust_monster
extern crate rust_monster;
use self::rust_monster::ga::ga_population::*;
use self::rust_monster::ga::ga_selectors::*;

// Alias GAIndividual
pub use self::rust_monster::ga::ga_core::GAIndividual as PolyminiGAIndividual;
// Alias GA
pub use self::rust_monster::ga::ga_core::GeneticAlgorithm as PolyminiGA;
// Alias GARandomCtx
pub use self::rust_monster::ga::ga_random::GARandomCtx as PolyminiRandomCtx;
//
//
//
use ::evaluation::*;
use ::instincts::*;
use ::uuid::*;

use std::any::Any;

// NOTE: Raw vs Fitness:
//
// For every other module we use the Raw score
// Internally, we use the Fitness Score in the GA as it is scaled using
// the Instincts tuning for the species
//
pub type PolyminiPopulationIter<'a, T> = GAPopulationRawIterator<'a, T>;

pub trait GAContext
{
    fn get_random_ctx(&mut self) -> &mut PolyminiRandomCtx;
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

    pub fn evaluate(&mut self, evaluators: &Vec<FitnessEvaluator>, instincts: &Vec<Instinct>)
    {
        for ref mut ind in &mut self.individuals.population().iter_mut()
        {
            let mut ctx = PolyminiEvaluationCtx::new_from(evaluators.clone(),
                                                          PolyminiFitnessAccumulator::new(instincts.clone()));
            ind.evaluate(&mut ctx);
        }
        self.individuals.force_sort();
        info!("Done Evaluating");
    }
}


// Genetic Algorithm Configuration
#[derive(Clone, Debug)]
pub struct PGAConfig
{
    pub max_generations: u32,
    pub population_size: u32,

    //  Percentage of individuals that pass from generation
    //  to generation
    pub percentage_elitism: f32,
    pub percentage_mutation: f32,

    // Evaluation Context
    pub fitness_evaluators: Vec<FitnessEvaluator>,

    // Genome Length
    pub genome_size: usize,

}
impl PGAConfig
{
    pub fn get_new_individuals_per_generation(&self) -> usize
    {
         (( 1.0 - self.percentage_elitism) * self.population_size as f32).floor() as usize
    }
}

pub struct PolyminiGeneticAlgorithm<T: PolyminiGAIndividual>
{
    current_generation: u32,
    population: PolyminiGeneration<T>,

    config: PGAConfig,
}
impl<T: PolyminiGAIndividual> PolyminiGeneticAlgorithm<T>
{
    pub fn new(pop: Vec<T>, uuid: PUUID, pgacfg: PGAConfig) -> PolyminiGeneticAlgorithm<T>
    {
        // TODO: Better seeds
        PolyminiGeneticAlgorithm {
                                   current_generation: 0,
                                   population: PolyminiGeneration::new(pop),
                                   config: pgacfg,
                                 }
    }

    pub fn new_with(pop: Vec<T>, pgacfg: PGAConfig) -> PolyminiGeneticAlgorithm<T>
    {
        PolyminiGeneticAlgorithm {
                                   current_generation: 0,
                                   population: PolyminiGeneration::new(pop),
                                   config: pgacfg,
                                 }

    }
    
    pub fn get_population(&self) -> &PolyminiGeneration<T>
    {
        &self.population
    }

    pub fn get_population_mut(&mut self) -> &mut PolyminiGeneration<T>
    {
        &mut self.population
    }

    pub fn evaluate_population(&mut self)
    {
        // TODO: Instincts should come from somehwere else like a config
        self.population.evaluate(&self.config.fitness_evaluators, &vec![ Instinct::Nomadic, Instinct::Basic, Instinct::Hoarding, Instinct::Herding, Instinct::Predatory ]);
    }

    pub fn population(&mut self) -> &mut GAPopulation<T>
    {
        &mut self.population.individuals
    }

    // Due to the nature of the GA, this step doesn't evaluate
    // it assumes an ordered list of individuals with their fitness set.
    // These responsibilities are offloaded to the 'evaluate' method of PolyminiGeneticAlgorithm
    pub fn step<C: 'static + GAContext>(&mut self, context: &mut C) -> i32
    {
        let mut new_individuals : Vec<T> = vec![];
        let mut roulette_selector = GARouletteWheelSelector::new(self.population.size());
        roulette_selector.update::<GAFitnessScoreSelection>(&mut self.population.individuals);
        // Build up new_individuals
        let new_num_individuals =  self.config.get_new_individuals_per_generation();

        for i in 0..new_num_individuals
        {
            let ind_1 = roulette_selector.select::<GAFitnessScoreSelection>(&self.population.individuals,
                                                                            &mut context.get_random_ctx());
            let ind_2 = roulette_selector.select::<GAFitnessScoreSelection>(&self.population.individuals,
                                                                            &mut context.get_random_ctx());

            let mut new_individual = *ind_1.crossover(ind_2, context);
            let mut_probability = context.get_random_ctx().gen_range(0.0, 1.0);
            if (mut_probability < self.config.percentage_mutation)
            {
                info!("Mutating Individual");
                new_individual.mutate(mut_probability, context);
            }
            new_individuals.push(new_individual);
        }

        // Copy over best individuals from previous gen
        let kept_individuals = self.population.size() - new_num_individuals; 


        let mut drain = self.population.individuals.drain_best_individuals(kept_individuals, GAPopulationSortBasis::Fitness);
        new_individuals.append(&mut drain);

        self.population.individuals = GAPopulation::new(new_individuals, GAPopulationSortOrder::HighIsBest);
        self.current_generation += 1;
        
        // TODO: Sort to avoid crashes in the Iterator, fix needed in rust-monster
        self.population.individuals.sort();
        self.current_generation as i32
    }

    pub fn done(&mut self) -> bool
    {
        // TODO: Configuration
        self.current_generation >= self.config.max_generations 
    }
}
