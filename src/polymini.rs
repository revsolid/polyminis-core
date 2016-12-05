use ::control::*;
use ::evaluation::*;
use ::genetics::*;
use ::instincts::*;
use ::morphology::*;
use ::physics::*;
use ::serialization::*;
use ::traits::*;
use ::types::*;
use ::uuid::*;

use std::any::Any;
use std::collections::HashMap;
use std::collections::hash_map::Entry;


pub struct Stats
{
    hp: i32,
    energy: i32,
    speed: usize,
    total_cells: usize,
    //TODO: combat_stats: CombatStatistics
}
impl Stats
{
    pub fn new(morph: &Morphology) -> Stats
    {
        let sp = Stats::calculate_speed_from(morph);
        let size = Stats::calculate_size_from(morph);
        Stats { hp: 0, energy: 0, speed: sp, total_cells: size }
    }
    fn calculate_speed_from(morph: &Morphology) -> usize
    {
        morph.get_traits_of_type(PolyminiTrait::PolyminiSimpleTrait(PolyminiSimpleTrait::SpeedTrait)).len()
    }
    fn calculate_size_from(morph: &Morphology) -> usize
    {
        morph.get_total_cells()
    }
}
impl Serializable for Stats
{
    fn serialize(&self, _: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();
        json_obj.insert("HP".to_owned(), self.hp.to_json());
        json_obj.insert("Energy".to_owned(), self.energy.to_json());
        json_obj.insert("Speed".to_owned(), (self.speed + 1).to_json());
        json_obj.insert("Size".to_owned(), self.total_cells.to_json());
        Json::Object(json_obj)
    }
}

pub struct Polymini
{
    uuid: PUUID,
    dead: bool,

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
        let corner = morphology.get_corner();

        let stats = Stats::new(&morphology);

        Polymini { uuid: uuid,
                   dead: false,
                   morph: morphology,
                   control: control,
                   physics: Physics::new_with_corner(uuid, dim, pos.0, pos.1, 0, corner),
                   stats: stats,
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
        if self.dead
        {
            return
        }
        self.control.sense(sp);
    }
    pub fn think_phase(&mut self)
    {
        if self.dead
        {
            return
        }
        self.control.think();
    }
    pub fn act_phase(&mut self, substep: usize, phys_world: &mut PhysicsWorld)
    {
        if self.dead
        {
            return
        }
        let actions = self.control.get_actions();
       
        debug!("Action List Len: {}", actions.len());
        // Feed them into other systems
        
        let speed = self.get_speed();
        self.physics.act_on(substep, speed, &actions, phys_world);

        let mut record_move_statistic = true;

        for action in actions
        {
            // TODO: Filter actions in a cleaner way
            match action
            {
                Action::MoveAction(_) =>
                {
                    if !self.physics.get_move_succeded() ||
                       !self.physics.get_acted() ||
                       !record_move_statistic 
                    {
                        continue
                    }
                    // Only record one action, even thou several actuators can output move actions
                    record_move_statistic = false;
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

    pub fn reset(&mut self, random_ctx: &mut PolyminiRandomCtx, placement_func: &PlacementFunction)
    {
        info!("Reseting {} - Had Fitness {}", self.uuid, self.fitness());
        self.physics.reset(random_ctx, placement_func);
        self.set_fitness(0.0);
        self.set_raw(0.0);
        self.fitness_statistics.clear();
    }

    pub fn die(&mut self, death_context: &DeathContext)
    {
        self.dead = true;
    }

    pub fn get_id(&self) -> PUUID 
    {
        self.uuid
    }

    pub fn consequence_physical(&mut self, world: &PhysicsWorld, substep: usize)
    {
        if self.dead
        {
            return
        }

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

    pub fn get_physics_mut(&mut self) -> &mut Physics
    {
        &mut self.physics
    }

    pub fn get_control(&self) -> &Control
    {
        &self.control
    }

    pub fn get_speed(&self) -> usize
    {
        self.stats.speed
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
        json_obj.insert("ID".to_owned(), self.get_id().to_json());

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATIC)
        {
            json_obj.insert("Morphology".to_owned(), self.get_morphology().serialize(ctx));
        }


        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_DYNAMIC)
        {
            json_obj.insert("Alive".to_owned(), Json::Boolean(!self.dead));
        }

        json_obj.insert("Control".to_owned(), self.get_control().serialize(ctx));

        json_obj.insert("Physics".to_owned(), self.get_physics().serialize(ctx));



        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATS)
        {
            json_obj.insert("Fitness".to_owned(), self.fitness().to_json());
            json_obj.insert("Raw".to_owned(), self.raw().to_json());
            json_obj.insert("Stats".to_owned(), self.stats.serialize(ctx));
            //
            let mut stats_json_obj = pmJsonObject::new();
            let mut stats_dict = HashMap::new();

            warn!("Len of Statistics: {}", self.fitness_statistics.len());
            for stat in &self.fitness_statistics
            {
                match stats_dict.entry(stat)
                {
                    Entry::Occupied(mut o) =>
                    {
                        let v = *o.get();
                        o.insert( v + 1 );
                    },
                    Entry::Vacant(mut o) =>
                    {
                        o.insert( 1 );
                    }
                }
            }

            for (k,v) in stats_dict.iter()
            {
                stats_json_obj.insert(k.to_string(), v.to_json());
            }

            
            json_obj.insert("FitnessStatistics".to_owned(), Json::Object(stats_json_obj));
        }

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
            
                self.morph.mutate(&mut creation_ctx.random_context, &creation_ctx.trans_table);
                let mut sensor_list = creation_ctx.default_sensors.clone();
                self.control.mutate(&mut creation_ctx.random_context, 
                                    sensor_list, self.morph.get_actuator_list());
                self.stats = Stats::new(&self.morph);
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

                self.fitness_statistics.push(FitnessStatistic::DistanceTravelled(self.physics.get_distance_moved() as u32));

                self.fitness_statistics.push(FitnessStatistic::TotalCells(self.stats.total_cells));

                let norm_pos = self.physics.get_normalized_pos();
                self.fitness_statistics.push(FitnessStatistic::FinalPosition((255.0*norm_pos.0) as u8,
                                                                             (255.0*norm_pos.1) as u8));

                if (self.dead)
                {
                    // TODO: Death reason, how long into the simulation before it died
                    self.fitness_statistics.push(FitnessStatistic::Died);
                }

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
