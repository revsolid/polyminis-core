use rust_monster::ga::ga_core::*;
use rust_monster::ga::ga_simple::*;
use rust_monster::ga::ga_random::*;

use std::any::Any;

use ::control::*;
use ::genetics::*;
use ::morphology::*;
use ::physics::*;


pub struct Species<'a>
{
    // Translation Table
    genetic_algorithm: SimpleGeneticAlgorithm<'a, Polymini>,
}


pub struct Statistics
{
    hp: i32,
    energy: i32,
}

pub struct Polymini
{
    uuid: usize,

    morph: Morphology,
    control: Control,
    physics: Physics,

    statistics: Statistics,

    fitness: f32
}
impl Polymini
{
    pub fn new(morphology: Morphology, control: Control) -> Polymini
    {
        Polymini { uuid: 0,
                   morph: morphology,
                   control: control,
                   physics: Physics::new(0, 0.0, 0.0, 0),
                   statistics: Statistics { hp: 0, energy: 0 },
                   fitness: 0.0 }
    }
    pub fn get_perspective(&self) -> Perspective
    {
        Perspective::new(self.uuid, self.physics.get_pos()) 
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

    pub fn get_id(&self) -> usize
    {
        self.uuid
    }

    pub fn consequence_physical(&mut self, world: &PhysicsWorld)
    {
        self.physics.update(world);
    }

    pub fn get_morphology(&self) -> &Morphology
    {
        &self.morph
    }

    pub fn get_physics(&self) -> &Physics
    {
        &self.physics
    }

}
impl GAIndividual for Polymini
{ 
    fn crossover(&self, other: &Polymini, random_ctx: &mut GARandomCtx) -> Box<Polymini>
    {
        let new_morphology = self.morph.crossover(&other.morph, random_ctx);
        let new_control = self.control.crossover(&other.control, random_ctx);

        Box::new(Polymini::new(new_morphology, new_control))
    }
    fn mutate(&mut self, _:f32, _: &mut GARandomCtx)
    {
        // Structural mutation should happen first
        //   morphology.mutate
        // Brain Mutation is self contained
        //   control.mutate
        // restart self (?)
    }
    fn evaluate(&mut self, _: &mut Any)
    {
        self.fitness;
    }
    fn fitness(&self) -> f32
    {
        self.fitness
    }
    fn set_fitness(&mut self, f: f32)
    {
        self.fitness = f;
    }
    fn raw(&self) -> f32
    {
        self.fitness
    }

    fn set_raw(&mut self, r: f32)
    {
        self.fitness = r;
    }
}
