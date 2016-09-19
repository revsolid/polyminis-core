use ::control::*;
use ::genetics::*;
use ::morphology::*;
use ::physics::*;
use ::serialization::*;
use ::uuid::*;

use std::any::Any;

pub struct Statistics
{
    hp: i32,
    energy: i32,
}

pub struct Polymini
{
    uuid: PUUID,

    morph: Morphology,
    control: Control,
    physics: Physics,

    statistics: Statistics,

    // TODO: Temporarily pub
    pub fitness: f32
}
impl Polymini
{
    pub fn new(morphology: Morphology, control: Control) -> Polymini
    {
        let uuid = PolyminiUUIDCtx::next();
        let dim = morphology.get_dimensions();
        Polymini { uuid: uuid,
                   morph: morphology,
                   control: control,
                   physics: Physics::new(uuid, dim, 1.0, 0.0, 0),
                   statistics: Statistics { hp: 0, energy: 0 },
                   fitness: 0.0 }
    }
    pub fn get_perspective(&self) -> Perspective
    {
        Perspective::new(self.uuid,
                         self.physics.get_pos(),
                         self.physics.get_orientation(),
                         self.physics.get_move_succeded())
    }
    pub fn sense_phase(&mut self, sp: &SensoryPayload)
    {
        self.control.sense(sp);
    }
    pub fn think_phase(&mut self)
    {
        self.control.think();
    }
    pub fn act_phase(&mut self, phys_world: &mut PhysicsWorld)
    {
        let actions = self.control.get_actions();

        // Feed them into other systems
        self.physics.act_on(actions, phys_world);
    }

    pub fn get_id(&self) -> PUUID 
    {
        self.uuid
    }

    pub fn consequence_physical(&mut self, world: &PhysicsWorld)
    {
        self.physics.update_state(world);
    }

    pub fn get_morphology(&self) -> &Morphology
    {
        &self.morph
    }

    pub fn get_physics(&self) -> &Physics
    {
        &self.physics
    }

    pub fn get_control(&self) -> &Control
    {
        &self.control
    }
}


// Polymini Serializable
//
impl Serializable for Polymini
{
    fn serialize(&self, ctx: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();
        json_obj.insert("id".to_string(), self.get_id().to_json());
        json_obj.insert("physics".to_string(), self.get_physics().serialize(ctx));
        json_obj.insert("morphology".to_string(), self.get_morphology().serialize(ctx));
        Json::Object(json_obj)
    }
}


// GA Individual
//
impl PolyminiGAIndividual for Polymini
{
    fn crossover(&self, other: &Polymini, ctx: &mut Any) -> Box<Polymini>
    {
        match ctx.downcast_mut::<PolyminiRandomCtx>()
        {
            Some(random_ctx) =>
            {
                let new_morphology = self.get_morphology().crossover(&other.get_morphology(), random_ctx);
                let new_control = self.get_control().crossover(&other.get_control(), random_ctx);
                Box::new(Polymini::new(new_morphology, new_control))
            },
            None =>
            {
                panic!("");
            }
        }
    }
    fn mutate(&mut self, _:f32, _: &mut Any)
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
