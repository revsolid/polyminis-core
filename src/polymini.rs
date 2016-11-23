use ::control::*;
use ::evaluation::*;
use ::genetics::*;
use ::instincts::*;
use ::morphology::*;
use ::physics::*;
use ::serialization::*;
use ::uuid::*;

use std::any::Any;


pub struct Stats
{
    hp: i32,
    energy: i32,
    //TODO: combat_stats: CombatStatistics
}

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
        

        //NOTE: Someone calling this function is opting into losing Random Context tracking
        //      that's ok but is on the caller, if tracking is required when creating a Polymini
        //      create Morphology and Control outside it and call new_with_control 
        let control = Control::new_from(morphology.get_sensor_list(), morphology.get_actuator_list(), 7,
                                        &mut RandomWeightsGenerator::new(&mut PolyminiRandomCtx::new_unseeded("TEMPORAL INDIVIDUAL".to_string())),
                                        &mut RandomWeightsGenerator::new(&mut PolyminiRandomCtx::new_unseeded("TEMPORAL INDIVIDUAL".to_string())));
         
        Polymini::new_with_control(pos, morphology, control)
    }
    pub fn new_with_control(pos: (f32, f32), morphology: Morphology, control: Control) -> Polymini
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

        debug!("Action List Len: {}", actions.len());
        // Feed them into other systems
        self.physics.act_on(&actions, phys_world);

        let mut move_already_recorded = false;
        for action in actions
        {
            // TODO: Filter actions in a cleaner way
            match action
            {
                Action::MoveAction(_) =>
                {
                    if !self.physics.get_move_succeded() ||
                        move_already_recorded
                    {
                        continue
                    }
                    move_already_recorded = true;
                },
                _ => {}
            }

            match FitnessStatistic::new_from_action(&action)
            {
                FitnessStatistic::NoOp =>
                {
                    debug!("NoOp Statistic - Skipping");
                },
                fitness_stat =>
                {
                    debug!("Inserting Fitness Statistic");
                    self.fitness_statistics.push(fitness_stat);
                }
            }
        }

        debug!("Fitness Statistics Len: {}", self.fitness_statistics.len());
    }

    pub fn reset(&mut self, random_ctx: &mut PolyminiRandomCtx)
    {
        // TODO - Important for individuals that survive
        // and handling Sim restarts
        info!("Reseting {} - Had Fitness {}", self.uuid, self.fitness());
        self.physics.reset( (random_ctx.gen_range(1, 100) as f32,
                             random_ctx.gen_range(1, 100) as f32) );
        self.set_fitness(0.0);
        self.set_raw(0.0);
        self.fitness_statistics.clear();
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
        json_obj.insert("id".to_owned(), self.get_id().to_json());
        json_obj.insert("Fitness".to_owned(), self.fitness().to_json());
        json_obj.insert("Raw".to_owned(), self.raw().to_json());

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATIC)
        {
            json_obj.insert("morphology".to_owned(), self.get_morphology().serialize(ctx));
        }

        json_obj.insert("control".to_owned(), self.get_control().serialize(ctx));

        json_obj.insert("physics".to_owned(), self.get_physics().serialize(ctx));
        Json::Object(json_obj)
    }
}


// GA Individual
//
impl PolyminiGAIndividual for Polymini
{
    fn crossover(&self, other: &Polymini, ctx: &mut Any) -> Box<Polymini>
    {
        info!("Crossing {} ({}) with {} ({})", self.uuid, self.fitness(), other.uuid, other.fitness());
        match ctx.downcast_mut::<PolyminiCreationCtx>()
        {
            Some(mut creation_ctx) =>
            {
                let new_morphology = self.get_morphology().crossover(&other.get_morphology(), &mut creation_ctx);
                let mut sensor_list = creation_ctx.default_sensors.clone();
                sensor_list.append(&mut new_morphology.get_sensor_list());

                let new_control = self.get_control().crossover(&other.get_control(), &mut creation_ctx.random_context,
                                                               sensor_list, new_morphology.get_actuator_list());
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
        info!("Mutating {}", self.uuid);
        match ctx.downcast_mut::<PolyminiCreationCtx>()
        {
            Some (creation_ctx) =>
            {
            // Structural mutation should happen first
                //self.morph.mutate(creation_ctx.random_context, &creation_ctx.trans_table);
                //self.control.mutate(creation_ctx.random_context, self.morph.get_sensor_list(), self.morph.get_actuator_list());
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
        info!("Evaluating individual {}", self.uuid);
        match ctx.downcast_mut::<PolyminiEvaluationCtx>()
        {
            Some (ctx) =>
            {
                debug!(" using {} statistics", self.fitness_statistics.len());

                self.fitness_statistics.push(FitnessStatistic::DistanceTravelled(self.physics.get_distance_moved()));

                ctx.evaluate(&self.fitness_statistics);
                self.set_raw(ctx.get_raw());
                //TODO: Get weights from somewhere
                self.set_fitness(ctx.get_raw());
            },
            None =>
            {
                panic!("");
            }
        }
        info!("  Result: {}", self.raw());
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
