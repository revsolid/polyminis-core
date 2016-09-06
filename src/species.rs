use ::polymini::*;
use ::genetics::*;

pub struct Species
{
    // Translation Table
    generation: PolyminiGeneration<Polymini> 
}
impl Species
{
    pub fn new(pop: Vec<Polymini>) -> Species
    {
        Species { generation: PolyminiGeneration::new(pop) }
    }

    pub fn get_generation(&self) -> &PolyminiGeneration<Polymini>
    {
        &self.generation
    }

    pub fn get_generation_mut(&mut self) -> &mut PolyminiGeneration<Polymini>
    {
        &mut self.generation
    }
}
