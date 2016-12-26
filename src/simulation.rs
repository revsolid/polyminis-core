use ::control::*;
use ::environment::*;
use ::physics::*;
use ::polymini::*;
use ::serialization::*;
use ::species::*;
use ::types::*;
use ::uuid::*;

// NOTE:
// Simulation vs Simulation Epoch
//
// A Simulation Epoch is a moment in evolution, while the Simulation is the process itself
// The Simulation keeps track of Database Connections or authentication context, while the Epoch is
// the actual evolution system
//
//
pub struct Simulation
{
    current_epoch: SimulationEpoch,
    epoch_num: usize,
}
impl Simulation
{
    pub fn new() -> Simulation
    {
        Simulation { current_epoch: SimulationEpoch::new(), epoch_num: 0 }
    }

    pub fn step(&mut self) -> bool
    {
        self.current_epoch.step();
        self.current_epoch.done()
    }

    pub fn get_epoch(&self) -> &SimulationEpoch
    {
        &self.current_epoch
    }

    pub fn get_epoch_mut(&mut self) -> &mut SimulationEpoch
    {
        &mut self.current_epoch
    }

    pub fn swap_epoch(&mut self, new_epoch: SimulationEpoch)
    {
        self.current_epoch = new_epoch;
    }

    pub fn advance_epoch(&mut self)
    {
        info!("Advancing Epoch - {} to {}", self.epoch_num, self.epoch_num + 1);
        self.epoch_num += 1;
        let new_epoch = self.current_epoch.advance();
        self.swap_epoch(new_epoch);
    }

}
pub struct SimulationEpoch
{
    environment: Environment,
    species: Vec<Species>,
    proportions: Vec<f32>,
    steps: usize,
    substeps: usize,
    max_steps: usize,
    restarts: usize,
    restarts_left: usize,
}
impl SimulationEpoch
{
    pub fn new() -> SimulationEpoch
    {
        SimulationEpoch { environment: Environment::new(2, vec![]), species: vec![], proportions: vec![], steps: 0, max_steps: 100, substeps: 4, restarts: 0, restarts_left: 0 }
    }

    pub fn new_with(environment: Environment, max_steps: usize) -> SimulationEpoch
    {
        SimulationEpoch::new_restartable(environment, max_steps, 0)
    }

    pub fn new_restartable(environment: Environment, max_steps: usize, restarts: usize) -> SimulationEpoch
    {
        SimulationEpoch { environment: environment, species: vec![], proportions: vec![], steps: 0,
                          max_steps: max_steps, substeps:4, restarts: restarts, restarts_left: restarts }
    }

    pub fn is_full(&self) -> bool
    {
        self.species.len() == self.environment.get_species_slots()
    }

   // pub fn add_object(&mut self, position: (f32, f32), dimensions: (u8, u8))
    pub fn add_object(&mut self, wo: WorldObject)
    {
        self.environment.add_object(wo);
    }

    pub fn add_species(&mut self, species: Species)
    {
        if self.is_full()
        {
            // Error ?
            return;
        }

        let mut sp = species;
        
        // Environment Registration
        debug!("Adding Species - Start Loop");
        for i in 0..sp.get_generation().size() 
        {
            debug!("{}", i);
            let ind = &mut sp.get_generation_mut().get_individual_mut(i);
            // An individual that can't be added to the environment is marked as
            // death to eliminate those genes from the pool as soon as possible
            if !self.environment.add_individual(ind)
            {
                ind.die(&DeathContext::new(DeathReason::Placement, 0));
            }
        }
        debug!("Adding Species - Done Loop");

        // Once fully registered we add them to the list of species
        self.species.push(sp);


        // Re-calculate proportions 
        let total_species_scores = self.species.iter().fold( 0.0, | mut accum, ref species |
        {
            accum += species.get_accum_score();
            accum
        }); 

        // Proportions
        self.proportions = self.species.iter().map( | ref species | species.get_accum_score() / total_species_scores as f32 ).collect();

        let new_prop = 1.0 / self.species.len() as f32;
    }
    
    pub fn get_species(&self) -> &Vec<Species>
    {
        &self.species
    }

    pub fn evaluate_species(&mut self)
    {
        for species in &mut self.species
        {
            species.evaluate();
        }
    }

    pub fn dump_species_random_ctx(&mut self)
    {
        for species in &mut self.species
        {
            species.dump_random_ctx();
        }
    }

    pub fn get_environment(&self) -> &Environment
    {
        &self.environment
    }

    pub fn restart(&mut self)
    {
        self.environment = self.environment.restart();
        for species in &mut self.species
        {
            species.restart();
        }

        let mut temp_species = vec![];
        temp_species.append(&mut self.species);
        for n_s in temp_species
        {
            self.add_species(n_s);
        }
    }

    // TODO: This should, in some way, destroy *self* epoch
    pub fn advance(&mut self) -> SimulationEpoch
    {
        debug!("Advancing Epoch - Species");
        for species in &mut self.species
        {
            species.advance_epoch();
        }

        let mut new_epoch_species = vec![];
        new_epoch_species.append(&mut self.species);

        // TODO: Advance the Environment's epoch and copy it over
        let mut new_epoch = SimulationEpoch::new_restartable(self.environment.advance_epoch(), self.max_steps, self.restarts);


        // Calculate Proportions of the species' fitness
        //
        let total_species_scores = self.species.iter().fold( 0.0, | mut accum, ref species |
        {
            accum += species.get_accum_score();
            accum
        }); 


        // Proportions
        new_epoch.proportions = self.species.iter().map( |ref species|  species.get_accum_score() / total_species_scores as f32 ).collect();

        debug!("Advancing Epoch - Reinserting Species");
        for n_s in new_epoch_species
        {
            new_epoch.add_species(n_s);
        }
        debug!("Advancing Epoch - Done Reinserting Species");

        new_epoch
    }

    pub fn step(&mut self)
    {
        let substep = self.steps % self.substeps;

        if self.steps == (self.max_steps * self.substeps) 
        {
            self.restart();
            self.restarts_left -= 1;
            self.steps = 0;
        }

        self.init_phase();
        self.sense_phase();
        self.think_phase();
        self.act_phase(substep);
        self.consequence_phase(substep);
        self.steps += 1;
    }

    pub fn done(&self) -> bool
    {
        self.steps == (self.max_steps * self.substeps) && self.restarts_left == 0
    }

    fn init_phase(&mut self)
    {
    }
    fn sense_phase(&mut self)
    {
        for s in 0..self.species.len()
        {
            let gen_size = self.species[s].get_generation().size();
            for i in 0..gen_size
            {
                let perspective;
                {
                    let polymini = self.species[s].get_generation().get_individual(i);
                    perspective = polymini.get_perspective();
                }
    
                let sensed = self.sense_for(&perspective);
                let mut p = self.species[s].get_generation_mut().get_individual_mut(i);
                p.sense_phase(&sensed);
            }
        }
    }
    fn think_phase(&mut self)
    {
        for s in &mut self.species
        {
            let generation = s.get_generation_mut();
            for i in 0..generation.size()
            {
                let mut polymini = generation.get_individual_mut(i);
                polymini.think_phase();
            }
        }
    }
    fn act_phase(&mut self, substep: usize)
    {
        for s in &mut self.species
        {
            let generation = s.get_generation_mut();
            for i in 0..generation.size()
            {
                let mut polymini = generation.get_individual_mut(i);
                polymini.act_phase(substep, &mut self.environment.physical_world);
            }
        }
    }
    fn consequence_phase(&mut self, substep: usize)
    {
        // Update environment based on the aftermath of the simulation

        /* Physics */
        self.environment.physical_world.step();

        for s in &mut self.species
        {
            let generation = s.get_generation_mut();
            for i in 0..generation.size()
            {
                let mut polymini = generation.get_individual_mut(i);
                polymini.consequence_physical(&self.environment.physical_world, substep);
            }
        }
        // After Physics is updated, each Polymini has data like
        // collisions and position, to be used by other systems
        // like combat

        // Combat

        // Energy consumption
        
        // GA Evaluation and Sort
    }
    fn sense_for(&self, perspective: &Perspective) -> SensoryPayload
    {
        let mut sp = SensoryPayload::new();
        // Fill the basic sensors
        sp.insert(SensorTag::PositionX, perspective.pos.0 / self.environment.dimensions.0);
        sp.insert(SensorTag::PositionY, perspective.pos.1 / self.environment.dimensions.1);

        sp.insert(SensorTag::LastMoveSucceded, if perspective.last_move_succeeded { 1.0 } else { 0.0 });

        sp.insert(SensorTag::Orientation, perspective.orientation.to_float());

        // Go through the environment and Polyminis filling up
        // the sensory payload
        sp
    }
}

//
//
impl Serializable for SimulationEpoch
{
    fn serialize(&self, ctx: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_DYNAMIC)
        {
            json_obj.insert("Step".to_owned(), self.steps.to_json());
        }

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATIC)
        {
            //
        }

        json_obj.insert("Environment".to_owned(), self.environment.serialize(ctx));


        let mut json_arr = pmJsonArray::new();
        for s in &self.species
        {
            json_arr.push(s.serialize(ctx));
        }
        json_obj.insert("Species".to_owned(), Json::Array(json_arr));

        Json::Object(json_obj)
    }
}


#[cfg(test)]
mod test
{
    extern crate env_logger;
    use super::*;

    use ::control::*;
    use ::environment::*;
    use ::morphology::*;
    use ::polymini::*;
    use ::serialization::*;
    use ::species::*;

    #[test]
    fn test_step()
    {
        //
        let mut s = SimulationEpoch::new();
        s.add_species(Species::new(vec![]));

        s.step();
    }

    #[test]
    fn test_step_2()
    {
        let _ = env_logger::init();
        let chromosomes = vec![[0, 0x09, 0x6A, 0xAD],
                               [0, 0x0B, 0xBE, 0xDA],
                               [0,    0, 0xBE, 0xEF],
                               [0,    0, 0xDB, 0xAD]];

        let p1 = Polymini::new(Morphology::new(&chromosomes, &TranslationTable::new()));

        let mut s = SimulationEpoch::new();
        s.add_species(Species::new(vec![p1]));
        s.step();
    }

    #[test]
    fn test_step_3()
    {
        let _ = env_logger::init();
        let chromosomes = vec![[0, 0x09, 0x6A, 0xAD],
                               [0, 0x0B, 0xBE, 0xDA],
                               [0,    0, 0xBE, 0xEF],
                               [0,    0, 0xDB, 0xAD]];

        let p1 = Polymini::new(Morphology::new(&chromosomes, &TranslationTable::new()));
        let mut s = SimulationEpoch::new();
        s.add_species(Species::new(vec![p1]));
        for _ in 0..10 
        {
            s.step();
        }
    }

    #[test]
    fn test_step_4()
    {
        let chromosomes = vec![[0, 0x09, 0x6A, 0xAD],
                               [0, 0x0B, 0xBE, 0xDA],
                               [0,    0, 0xBE, 0xEF],
                               [0,    0, 0xDB, 0xAD]];

        let p1 = Polymini::new(Morphology::new(&chromosomes, &TranslationTable::new()));
        let mut s = SimulationEpoch::new();
        s.add_species(Species::new(vec![p1]));
        s.add_object(WorldObject::new_static_object((10.0, 2.0), (1, 1)));
        for _ in 0..10 
        {
            s.step();
            println!("{}", s.serialize(&mut SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_STATIC |
                                                                             PolyminiSerializationFlags::PM_SF_DYNAMIC)));
        }
    }

    #[test]
    fn test_step_double_coll()
    {
        let _ = env_logger::init();
        let chromosomes = vec![[0, 0x09, 0x6A, 0xAD],
                               [0, 0x0B, 0xBE, 0xDA],
                               [0,    0, 0xBE, 0xEF],
                               [0,    0, 0xDB, 0xAD]];

        let chromosomes2 = vec![[0, 0x09, 0x6A, 0xAD],
                                [0, 0x0B, 0xBE, 0xDA],
                                [0,    0, 0xBE, 0xEF],
                                [0,    0, 0xDB, 0xAD]];

        let p1 = Polymini::new_at((21.0, 20.0), Morphology::new(&chromosomes, &TranslationTable::new()));
        let p2 = Polymini::new_at((17.0, 20.0), Morphology::new(&chromosomes2, &TranslationTable::new()));

        println!("{:?}", p1.get_morphology());
        println!(">> {:?}", p1.get_physics().get_pos());
        println!("{:?}", p2.get_morphology());
        println!(">> {:?}", p2.get_physics().get_pos());
        let mut s = SimulationEpoch::new();
        s.add_species(Species::new(vec![p1, p2]));
        s.add_object(WorldObject::new_static_object((20.0, 22.0), (1, 1)));
        for _ in 0..10 
        {
            s.step();
        }
    }
}
