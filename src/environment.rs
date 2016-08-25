use ::polymini::*;


struct Environment
{
    simulation: Simulation,
}

struct World
{
    // Physics Manager
}
impl World
{
    pub fn apply(&self, _: &ActionList)
    {}
}

struct Simulation
{
    world: World,
}
impl Simulation
{
    fn step(&mut self, generation: &mut PolyminiGeneration)
    {
        self.environment_setup(generation);
        self.sense_phase(generation);
        self.think_phase(generation);
        self.act_phase(generation);
        self.environment_update(generation);
    }
    fn environment_setup(&self, _: &mut PolyminiGeneration)
    {
        // Set up World Sensable information once
    }
    fn sense_phase(&self, generation: &mut PolyminiGeneration)
    {
        for polymini in &mut generation.individuals.population().iter_mut()
        {
            let sp = self.sense_for(&polymini);
            polymini.sense_phase(&sp);
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
            self.world.apply(&al);
        } 
    }
    fn environment_update(&self, _: &mut PolyminiGeneration)
    {
        // Update environment based on the aftermath of the simulation
    }

    //
    fn sense_for(&self, _: &Polymini) -> SensoryPayload
    {
        SensoryPayload {}
    }

    fn actions_for(&self, _: &Polymini) -> ActionList
    {
        ActionList {}
    }
}
