use rust_monster::ga::ga_core::*;
use rust_monster::ga::ga_population::*;
use rust_monster::ga::ga_simple::*;

use ::morphology::*;

pub struct SensoryPayload{}
pub struct ActionList {}


pub struct Species
{
    // Translation Table
    genetic_algorithm: SimpleGeneticAlgorithm<Polymini>, 
}


struct Splice {} 
struct Trait {}

pub struct Sensor
{
}
pub struct Actuator
{
}
pub struct Control
{
    // SensorList
    // ActuatorList
    // NN
}
impl Control
{
    pub fn sense(&self, _: &SensoryPayload)
    {
        // Feed SensoryPayload into sensors
        // Copy values from sensors to input layer of NN
    }
    pub fn think(&self)
    {
        // Feedforward NN
        // Copy values from output layer into Actuators
    }
    pub fn act(&self, _: &mut ActionList)
    {
        // Get actions from Actuators
        // Copy actions into ActionList
    }
}

pub struct Physics {}

pub struct Statistics
{
    hp: i32,
    energy: i32,
}

pub struct Polymini
{
    morph: Morphology,
    control: Control,
    physics: Physics,
    statistics: Statistics,

    fitness: f32
}
impl Polymini
{
    pub fn new(f:f32) -> Polymini
    {
        Polymini { control: Control {},
                   morph: Morphology::new(),
                   physics: Physics {},
                   statistics: Statistics { hp: 0, energy: 0 },
                   fitness: f}
    }
    pub fn sense_phase(&mut self, sp: &SensoryPayload)
    {
        self.control.sense(sp);
    }
    pub fn think_phase(&mut self)
    {
        self.control.think();
    }
    pub fn act_phase(&mut self, al: &mut ActionList)
    {
        self.control.act(al);
    }
}
impl GAIndividual for Polymini
{ 
    fn crossover(&self, _: &Polymini) -> Box<Polymini>
    {
        Box::new(Polymini::new(0.0))
    }
    fn mutate(&mut self, _:f32)
    {
        // Structural mutation should happen first
        //   morphology.mutate
        // Brain Mutation is self contained
        //   control.mutate
        // restart self (?)
    }

    fn evaluate(&mut self) -> f32
    {
        self.fitness 
    }
    fn fitness(&self) -> f32
    {
        self.fitness
    }
    fn set_fitness(&mut self, f: f32)
    {
        self.fitness = f
    }
    fn raw(&self) -> f32
    {
        0.0
    }
}

pub struct PolyminiGeneration<'a>
{
    pub individuals: &'a mut GAPopulation<Polymini>
}
