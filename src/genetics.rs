//
// LAYERING - THIS IS THE ONLY PLACE ALLOWED TO INCLUDE rust_monster (except for random)
extern crate rust_monster;
use self::rust_monster::ga::ga_population::*;
use self::rust_monster::ga::ga_selectors::*;

// Alias GAIndividual
pub use self::rust_monster::ga::ga_core::GAIndividual as PolyminiGAIndividual;
// Alias GA
pub use self::rust_monster::ga::ga_core::GeneticAlgorithm as PolyminiGA;
//
//
//
use ::evaluation::*;
use ::instincts::*;
use ::serialization::*;
use ::uuid::*;

pub use ::random::PolyminiRandomCtx as PolyminiRandomCtx;

use std::any::Any;
use std::collections::HashMap;

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

    pub fn evaluate(&mut self, evaluators: &Vec<FitnessEvaluator>, instincts: &Vec<Instinct>,
                    instinct_weights: &HashMap<Instinct,f32>, accumulates: bool)
    {
        for ref mut ind in &mut self.individuals.population().iter_mut()
        {
            let mut ctx = PolyminiEvaluationCtx::new_from(evaluators.clone(),
                                                          PolyminiFitnessAccumulator::new(instincts.clone()), instinct_weights.clone(), accumulates);
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
    pub population_size: u32,

    //  Percentage of individuals that pass from generation
    //  to generation
    pub percentage_elitism: f32,
    pub percentage_mutation: f32,

    // Evaluation Context
    pub accumulates_over: bool, // Accumulate the result of the evaulation over several calls to evaluate
    pub fitness_evaluators: Vec<FitnessEvaluator>,

    // Genome Length
    pub genome_size: usize,

}
impl PGAConfig
{
    pub fn defaults() -> PGAConfig
    {
        PGAConfig
        {
            population_size: 100,
            percentage_elitism:  0.8,
            percentage_mutation: 0.2,
            fitness_evaluators: vec![],
            accumulates_over: false,
            genome_size: 4,
        }
    }
    pub fn get_new_individuals_per_generation(&self) -> usize
    {
         (( 1.0 - self.percentage_elitism) * self.population_size as f32).floor() as usize
    }
}
impl Serializable for PGAConfig
{
    fn serialize(&self, ctx: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();
        json_obj.insert("PopulationSize".to_owned(), self.population_size.to_json());
        json_obj.insert("PercentageElitism".to_owned(), self.percentage_elitism.to_json());
        json_obj.insert("PercentageMutation".to_owned(), self.percentage_mutation.to_json());
        json_obj.insert("GenomeSize".to_owned(), self.genome_size.to_json());

        /*json_obj.insert("InstinctWeights".to_owned(), 
            Json::Object(
            {
                let mut iw_json_obj = pmJsonObject::new();
                for (k,v) in &self.instinct_weights
                {
                    iw_json_obj.insert(k.to_string(), v.to_json());
                }
                iw_json_obj
            }));*/

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_DB)
        {
            json_obj.insert("FitnessEvaluators".to_owned(),
                            Json::Array(self.fitness_evaluators.iter().map(
                            {
                                |fe|
                                {
                                    fe.serialize(ctx)  
                                }
                            }).collect()));
        }
        Json::Object(json_obj)
    }
}
impl Deserializable for PGAConfig
{
    fn new_from_json(json: &Json, ctx: &mut SerializationCtx) -> Option<PGAConfig> 
    {
        match *json
        {
            Json::Object(ref json_obj) =>
            {

                if !JsonUtils::verify_has_fields(&json_obj, &vec!["PopulationSize".to_owned(), "GenomeSize".to_owned(), "PercentageElitism".to_owned(), "PercentageMutation".to_owned()])
                {
                   // The Verify should've logged what is missing we can return 
                   return None
                }
                let ps = json_obj.get("PopulationSize").unwrap().as_u64().unwrap() as u32;
                let gs = json_obj.get("GenomeSize").unwrap().as_u64().unwrap() as usize;
                let pe = json_obj.get("PercentageElitism").unwrap().as_f64().unwrap() as f32;
                let pm = json_obj.get("PercentageMutation").unwrap().as_f64().unwrap() as f32;
                let fe = match json_obj.get("FitnessEvaluators")
                {
                    Some(json_arr) =>
                    {
                        json_arr.as_array().unwrap().iter().map(
                        |e|
                        {
                            FitnessEvaluator::new_from_json(e, &mut SerializationCtx::new()).unwrap()
                        }).collect()
                    }
                    _ =>
                    {
                        vec![]
                    }
                };

                let ao = json_obj.get("AccumulatesOver").unwrap_or(&Json::Boolean(false)).as_boolean().unwrap();
 
                Some(PGAConfig { population_size: ps,
                                 percentage_elitism: pe, fitness_evaluators: fe, accumulates_over: ao,
                                 percentage_mutation: pm, genome_size: gs })
            },
            _ =>
            {
                None
            }
        }
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

    pub fn get_config(&self) -> &PGAConfig
    {
        &self.config
    }

    pub fn change_config(&mut self, config: PGAConfig)
    {
        self.config = config;
        self.current_generation = 0;
    }
    
    pub fn get_population(&self) -> &PolyminiGeneration<T>
    {
        &self.population
    }

    pub fn get_population_mut(&mut self) -> &mut PolyminiGeneration<T>
    {
        &mut self.population
    }

    pub fn evaluate_population(&mut self, instinct_weights: &HashMap<Instinct, f32>)
    {
        // TODO: Instincts should come from somehwere else like a config
        self.population.evaluate(&self.config.fitness_evaluators,
                                 &vec![ Instinct::Nomadic, Instinct::Basic, Instinct::Hoarding, Instinct::Herding, Instinct::Predatory ],
                                 instinct_weights, self.config.accumulates_over);
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
        // NOTE: 'done' doesn't make sense for our use case, our algorithm is never done
        false
    }
}


#[cfg(test)]
mod test
{
    use super::*;
    use ::evaluation::*;
    use ::instincts::*;
    use ::serialization::*;
    use ::uuid::*;

    use std::collections::HashMap;

    #[test]
    fn test_pga_serialization()
    {

        let evaluators = vec![ FitnessEvaluator::OverallMovement { weight: 2.5 },
                               FitnessEvaluator::DistanceTravelled { weight: 2.0 },
                               FitnessEvaluator::Shape { weight: 5.0 }];
        let cfg = PGAConfig { population_size: 50,
                              percentage_elitism: 0.11, percentage_mutation: 0.12, fitness_evaluators: evaluators, accumulates_over: false,
                              genome_size: 8 };
        let ser_ctx = &mut SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DB);
                              
        let json_1 = cfg.serialize(ser_ctx);
        let cfg_prime = PGAConfig::new_from_json(&json_1, ser_ctx).unwrap();
        let json_2 = cfg_prime.serialize(ser_ctx);

        assert_eq!(json_1.pretty().to_string(), json_2.pretty().to_string());
    }
}
