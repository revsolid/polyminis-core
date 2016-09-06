use ::control::*;
use ::physics::*;
use ::species::*;

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
    pub fn step(&mut self)
    {
        self.environment_setup();
        self.sense_phase();
        self.think_phase();
        self.act_phase();
        self.consequence();
    }
    pub fn add(&mut self, species: Species)
    {
        // Do something?
        self.species.push(species);
    }

    fn environment_setup(&self)
    {
        // Set up World Sensable information once
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

    use ::species::*;

    #[test]
    fn test_step()
    {
        //
        let mut s = Simulation::new();
        s.add(Species::new( vec![] ));

        s.step();
    }
}
