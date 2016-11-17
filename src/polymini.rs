use ::control::*;
use ::genetics::*;
use ::instincts::*;
use ::morphology::*;
use ::physics::*;
use ::serialization::*;
use ::uuid::*;

use std::any::Any;


mod Evaluation
{
    use ::actuators::*;
    use ::instincts::*;
    use std::collections::HashMap;
    pub enum FitnessStatistic
    {
        NoOp,
        Moved,
        ConsumedFoodSource,
        Died,
    }
    impl FitnessStatistic
    {
        pub fn new_from_action(action: &Action) -> FitnessStatistic
        {
            match *action
            {
                Action::MoveAction(_) =>
                {
                    return FitnessStatistic::Moved
                }
                _ => 
                {
                    return FitnessStatistic::NoOp
                }
            }
        }
    }

    pub enum FitnessEvaluator
    {
    }
    impl FitnessEvaluator
    {
        pub fn evaluate(&mut self, statistics: &Vec<FitnessStatistic>) -> (Instinct, f32)
        {
            (Instinct::Basic, 0.0)
        }
    }

    pub struct PolyminiEvaluationCtx
    {
        evaluators: Vec<FitnessEvaluator>,
        accumulator: PolyminiFitnessAccumulator,
    }
    impl PolyminiEvaluationCtx
    {
        pub fn evaluate(&mut self, statistics: &Vec<FitnessStatistic>)
        {
            self.evaluators.iter_mut().fold(&mut self.accumulator,
                                            |accum, ref mut evaluator|
                                            {
                                                let v = evaluator.evaluate(statistics);
                                                accum.add(&v.0, v.1);
                                                accum
                                            });
        }
    }

    pub struct PolyminiFitnessAccumulator
    {
        accumulated_by_instinct: HashMap<Instinct, f32>,
    }
    impl PolyminiFitnessAccumulator
    {
        pub fn new(instincts: Vec<Instinct>) -> PolyminiFitnessAccumulator
        {
            let mut map = HashMap::new();

            assert!(instincts.len() > 0, "No instincts will yield no evolution");

            for i in &instincts
            {
                map.insert(*i, 0.0); 
            }

            PolyminiFitnessAccumulator { accumulated_by_instinct: map }
        }
        pub fn add(&mut self, instinct: &Instinct, v: f32)
        {
            let new_v;
            match self.accumulated_by_instinct.get(instinct)
            {
                Some(accum) => { new_v = accum + v; },
                None => { panic!("Incorrectly Initialized Accumulator") }
            }

            self.accumulated_by_instinct.insert(*instinct, new_v);
        }
    }
}



pub struct Stats
{
    hp: i32,
    energy: i32,
    //TODO: combat_stats: CombatStatistics
}


use self::Evaluation::*;
pub struct Polymini
{
    uuid: PUUID,

    morph: Morphology,
    control: Control,
    physics: Physics,
    stats: Stats,

    // Statistics to evaluate the creature
    fitness_statistics: Vec<FitnessStatistic>,

    // Species-Agnostic Score
    raw_score: f32,

    // raw_score scaled by the Instintcts Tuning  (aka. Fitness)
    species_weighted_fitness: f32,
}
impl Polymini
{
    pub fn new(morphology: Morphology) -> Polymini
    {
        Polymini::new_at((0.0, 0.0), morphology)
    }
    pub fn new_at(pos: (f32, f32), morphology: Morphology) -> Polymini
    {
        //Build up the control
        // TODO: Random Context :(
        let control = Control::new_from(morphology.get_sensor_list(), morphology.get_actuator_list(), 7,
                                        &mut RandomWeightsGenerator::new(&mut PolyminiRandomCtx::new_unseeded("TEMPORAL INDIVIDUAL".to_string())),
                                        &mut RandomWeightsGenerator::new(&mut PolyminiRandomCtx::new_unseeded("TEMPORAL INDIVIDUAL".to_string())));
         
        Polymini::new_with_control(pos, morphology, control)
    }
    fn new_with_control(pos: (f32, f32), morphology: Morphology, control: Control) -> Polymini
    {
        let uuid = PolyminiUUIDCtx::next();
        let dim = morphology.get_dimensions();
        Polymini { uuid: uuid,
                   morph: morphology,
                   control: control,
                   physics: Physics::new(uuid, dim, pos.0, pos.1, 0),
                   stats: Stats { hp: 0, energy: 0 },
                   fitness_statistics: vec![],
                   raw_score: 0.0,
                   species_weighted_fitness: 0.0 }

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
        self.physics.act_on(&actions, phys_world);


        for action in actions
        {
            match FitnessStatistic::new_from_action(&action)
            {
                FitnessStatistic::NoOp => {},
                fitness_stat => { self.fitness_statistics.push(fitness_stat); }
            }
        }
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

    pub fn add_global_statistics(&mut self, global_stats: &mut Vec<FitnessStatistic>)
    {
        self.fitness_statistics.append(global_stats);
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

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATIC)
        {
            json_obj.insert("morphology".to_string(), self.get_morphology().serialize(ctx));
        }

        json_obj.insert("physics".to_string(), self.get_physics().serialize(ctx));
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
                let new_control = self.get_control().crossover(&other.get_control(), random_ctx, new_morphology.get_sensor_list(),
                                                               new_morphology.get_actuator_list());
                Box::new(Polymini::new_with_control((0.0, 0.0), new_morphology, new_control))
            },
            None =>
            {
                panic!("Invalid Crossover Context");
            }
        }
    }
    fn mutate(&mut self, _:f32, ctx: &mut Any)
    {
        match ctx.downcast_mut::<PolyminiRandomCtx>()
        {
            Some (random_ctx) =>
            {
            // Structural mutation should happen first
                self.morph.mutate(random_ctx, &TranslationTable::new());
                self.control.mutate(random_ctx, self.morph.get_sensor_list(), self.morph.get_actuator_list());
            },
            None =>
            {
                panic!("Invalid Mutation Context");
            }
        }
        // restart self (?)
    }
    fn evaluate(&mut self, ctx: &mut Any)
    {
        match ctx.downcast_mut::<PolyminiEvaluationCtx>()
        {
            Some (ctx) =>
            {
                let raw_value = 0.0;
                self.set_raw(raw_value);
            },
            None =>
            {
                panic!("");
            }
        }
    }
    fn fitness(&self) -> f32
    {
        self.species_weighted_fitness
    }
    fn set_fitness(&mut self, f: f32)
    {
        self.species_weighted_fitness = f;
    }
    fn raw(&self) -> f32
    {
        self.raw_score
    }

    fn set_raw(&mut self, r: f32)
    {
        self.raw_score = r;
    }
}
