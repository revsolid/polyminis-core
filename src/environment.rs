use ::control::*;
use ::genetics::*;
use ::physics::*;
use ::polymini::*;

use rust_monster::ga::ga_population::*;

struct Environment
{
    simulation: Simulation,
}

struct Simulation
{
    physical_world: PhysicsWorld,
}
impl Simulation
{
    fn step(&mut self, generation: &mut PolyminiGeneration)
    {
        self.environment_setup(generation);
        self.sense_phase(generation);
        self.think_phase(generation);
        self.act_phase(generation);
        self.consequence(generation);
    }
    fn environment_setup(&self, _: &mut PolyminiGeneration)
    {
        // Set up World Sensable information once
    }
    fn sense_phase(&self, generation: &mut PolyminiGeneration)
    {

        for i in 0..generation.individuals.size()
        {
            let sensed;
            // TODO: Some better abstraction over rust-monster stuff would be great
            {
                let polymini = generation.individuals.individual(i, GAPopulationSortBasis::Raw);
                sensed = self.sense_for(&polymini.get_perspective());
            }

            //TODO: inviduals.individual_mut is needed
            //let mut k: &mut Polymini = &mut generation.individuals.individual(i, GAPopulationSortBasis::Raw);
        }
    }
    fn think_phase(&self, generation: &mut PolyminiGeneration)
    {
        for polymini in &mut generation.individuals.population().iter_mut()
        {
            polymini.think_phase();
        } 
    }
    fn act_phase(&self, generation: &mut PolyminiGeneration) 
    {
        for polymini in &mut generation.individuals.population().iter_mut()
        {
            let mut al = self.actions_for(&polymini);
            polymini.act_phase(&mut al);
        } 
    }
    fn consequence(&mut self, _: &mut PolyminiGeneration)
    {
        // Update environment based on the aftermath of the simulation

        /* Physics */
        self.physical_world.step();
        /* Loop through the generation and update their physics situation */ 
        // After physics, internal polymini state will be updated

        // Combat
        
        // Energy consumption
    }

    fn sense_for(&self, _: &Perspective) -> SensoryPayload
    {
        // Go through the environment and Polyminis filling up
        // the sensory payload
        SensoryPayload::new()
    }

    fn actions_for(&self, _: &Polymini) -> ActionList
    {
        ActionList {}
    }
}
