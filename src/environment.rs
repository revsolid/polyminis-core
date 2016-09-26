use ::control::*;
use ::physics::*;
use ::species::*;
use ::uuid::*;

struct Environment
{
    simulation: Simulation,
}

pub struct Simulation
{
    physical_world: PhysicsWorld,
    species: Vec<Species>,
}
impl Simulation
{
    pub fn new() -> Simulation
    {
        Simulation { physical_world: PhysicsWorld::new(), species: vec![] } 
    }

    pub fn add_object(&mut self, position: (f32, f32), dimensions: (u8, u8))
    {
        self.physical_world.add_object(PolyminiUUIDCtx::next(),
                                       position,
                                       dimensions);

    }

    pub fn add_species(&mut self, species: Species)
    {
        // Register all individuals in that species to the respective worlds
        //
        let _ = species.get_generation().size();
        
        // Physics Registration
        for ind in species.get_generation().iter()
        {
            println!(">>> Adding Invidivudal to Physical World");
            self.physical_world.add(ind.get_physics(), ind.get_morphology());
        }
        self.physical_world.step();

        // Once fully registered we add them to the list of species
        self.species.push(species);
    }

    pub fn step(&mut self)
    {
        self.environment_setup();
        self.sense_phase();
        self.think_phase();
        self.act_phase();
        self.consequence();
    }

    fn environment_setup(&self)
    {
        // Set up World 
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
                polymini.act_phase(&mut self.physical_world);
            }
        }
    }
    fn consequence(&mut self)
    {
        // Update environment based on the aftermath of the simulation

        /* Physics */
        self.physical_world.step();

        for s in &mut self.species
        {
            let generation = s.get_generation_mut();
            for i in 0..generation.size()
            {
                let mut polymini = generation.get_individual_mut(i);
                polymini.consequence_physical(&self.physical_world);
            }
        }
        // After Physics is updated, each Polymini has data like
        // collisions and position, to be used by other systems
        // like combat

        // Combat

        // Energy consumption
    }
    fn sense_for(&self, _: &Perspective) -> SensoryPayload
    {
        let sp = SensoryPayload::new();
        // Go through the environment and Polyminis filling up
        // the sensory payload
        sp
    }
}


#[cfg(test)]
mod test
{
    use super::*;

    use ::control::*;
    use ::morphology::*;
    use ::polymini::*;
    use ::species::*;

    #[test]
    fn test_step()
    {
        //
        let mut s = Simulation::new();
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

        let p1 = Polymini::new(Morphology::new(chromosomes),
                               Control::new());
        println!(">> {:?}", p1.get_physics().get_pos());

        let mut s = Simulation::new();
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

        let p1 = Polymini::new(Morphology::new(chromosomes),
                               Control::new());
        println!(">> {:?}", p1.get_physics().get_pos());
        let mut s = Simulation::new();
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

        let p1 = Polymini::new(Morphology::new(chromosomes),
                               Control::new());
        println!("{:?}", p1.get_morphology());
        println!(">> {:?}", p1.get_physics().get_pos());
        let mut s = Simulation::new();
        s.add_species(Species::new(vec![p1]));
        s.add_object((10.0, 2.0), (1, 1));
        for _ in 0..10 
        {
            s.step();
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

        let p1 = Polymini::new_at((1.0, 0.0), Morphology::new(chromosomes),
                               Control::new());
        let p2 = Polymini::new_at((-3.0, 0.0), Morphology::new(chromosomes2),
                               Control::new());

        println!("{:?}", p1.get_morphology());
        println!(">> {:?}", p1.get_physics().get_pos());
        println!("{:?}", p2.get_morphology());
        println!(">> {:?}", p2.get_physics().get_pos());
        let mut s = Simulation::new();
        s.add_species(Species::new(vec![p1, p2]));
        s.add_object((10.0, 2.0), (1, 1));
        for _ in 0..10 
        {
            s.step();
        }
        assert_eq!(0,1);
    }


}
