use ::control::*;
use ::environment::*;
use ::physics::*;
use ::polymini::*;
use ::serialization::*;
use ::species::*;
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
        info!("Advancing Epoch");
        let new_epoch = self.current_epoch.advance();
        self.swap_epoch(new_epoch);
    }

}
pub struct SimulationEpoch
{
    environment: Environment,
    species: Vec<Species>,
    steps: usize,
    max_steps: usize,
}
impl SimulationEpoch
{
    pub fn new() -> SimulationEpoch
    {
        SimulationEpoch { environment: Environment::new(2, vec![]), species: vec![], steps: 0, max_steps: 100 } 
    }

    pub fn new_with(environment: Environment, max_steps: usize) -> SimulationEpoch
    {
        SimulationEpoch { environment: environment, species: vec![], steps: 0,
                          max_steps: max_steps } 
    }

    pub fn is_full(&self) -> bool
    {
        self.species.len() == self.environment.get_species_slots()
    }

    pub fn add_object(&mut self, position: (f32, f32), dimensions: (u8, u8))
    {
        self.environment.add_object(position, dimensions);
    }

    pub fn add_species(&mut self, species: Species)
    {
        if self.is_full()
        {
            // Error ?
            return;
        }
        
        // Environment Registration
        for ind in species.get_generation().iter()
        {
            self.environment.add_individual(ind);
        }
        self.environment.physical_world.finish_adding();

        // Once fully registered we add them to the list of species
        self.species.push(species);
    }

    pub fn evaluate_species(&mut self)
    {
        for species in &mut self.species
        {
            species.evaluate();
        }
    }

    // TODO: This should, in some way, destroy *self* epoch
    pub fn advance(&mut self) -> SimulationEpoch
    {
        for species in &mut self.species
        {
            species.advance_epoch();
        }

        let mut new_epoch_species = vec![];
        new_epoch_species.append(&mut self.species);

        // TODO: Advance the Environment's epoch and copy it over
        SimulationEpoch { environment: Environment::new(self.environment.get_species_slots(), self.environment.default_sensors.clone()), species: new_epoch_species, steps: 0, max_steps: 0 }
    }

    pub fn step(&mut self)
    {
        self.init_phase();
        self.sense_phase();
        self.think_phase();
        self.act_phase();
        self.consequence_phase();
        self.steps += 1;
    }

    pub fn done(&self) -> bool
    {
        self.steps == self.max_steps
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
    fn act_phase(&mut self)
    {
        for s in &mut self.species
        {
            let generation = s.get_generation_mut();
            for i in 0..generation.size()
            {
                let mut polymini = generation.get_individual_mut(i);
                polymini.act_phase(&mut self.environment.physical_world);
            }
        }
    }
    fn consequence_phase(&mut self)
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
                polymini.consequence_physical(&self.environment.physical_world);
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
            json_obj.insert("step".to_owned(), self.steps.to_json());
        }

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATIC)
        {
            //
        }

        json_obj.insert("environment".to_owned(), self.environment.serialize(ctx));


        let mut json_arr = pmJsonArray::new();
        for s in &self.species
        {
            json_arr.push(s.serialize(ctx));
        }
        json_obj.insert("species".to_owned(), Json::Array(json_arr));

        Json::Object(json_obj)
    }
}


#[cfg(test)]
mod test
{
    use super::*;

    use ::control::*;
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
        s.add_object((10.0, 2.0), (1, 1));
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
        let chromosomes = vec![[0, 0x09, 0x6A, 0xAD],
                               [0, 0x0B, 0xBE, 0xDA],
                               [0,    0, 0xBE, 0xEF],
                               [0,    0, 0xDB, 0xAD]];

        let chromosomes2 = vec![[0, 0x09, 0x6A, 0xAD],
                                [0, 0x0B, 0xBE, 0xDA],
                                [0,    0, 0xBE, 0xEF],
                                [0,    0, 0xDB, 0xAD]];

        let p1 = Polymini::new_at((1.0, 0.0), Morphology::new(&chromosomes, &TranslationTable::new()));
        let p2 = Polymini::new_at((-3.0, 0.0), Morphology::new(&chromosomes2, &TranslationTable::new()));

        println!("{:?}", p1.get_morphology());
        println!(">> {:?}", p1.get_physics().get_pos());
        println!("{:?}", p2.get_morphology());
        println!(">> {:?}", p2.get_physics().get_pos());
        let mut s = SimulationEpoch::new();
        s.add_species(Species::new(vec![p1, p2]));
        s.add_object((10.0, 2.0), (1, 1));
        for _ in 0..10 
        {
            s.step();
        }
    }
}
