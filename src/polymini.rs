use ::control::*;
use ::evaluation::*;
use ::genetics::*;
use ::instincts::*;
use ::morphology::*;
use ::ph::*;
use ::physics::*;
use ::serialization::*;
use ::thermal::*;
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
const BASE_LINE_TMP: (f32, f32) = (-0.1, 0.2);
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
        morph.get_traits_of_type(PolyminiTrait::PolyminiSimpleTrait(TraitTag::SpeedTrait)).len()
    }
    fn calculate_size_from(morph: &Morphology) -> usize
    {
        morph.get_total_cells()
    }
    fn calculate_temperature_range(morph: &Morphology) -> (f32, f32)
    {
        // What is the starting polymini temperature?
        // TODO: Make this much more data driven
        let mut h_resist = 0.05 *
                           morph.get_traits_of_type(PolyminiTrait::PolyminiSimpleTrait(
                                                      TraitTag::ThermalHotResist)).len() as f32;
        let mut c_resist = -0.05 *
                           morph.get_traits_of_type(PolyminiTrait::PolyminiSimpleTrait(
                                                      TraitTag::ThermalColdResist)).len() as f32;

        let min_tmp = (BASE_LINE_TMP.0 + c_resist).max(0.0);
        let max_tmp = (BASE_LINE_TMP.1 + h_resist).min(1.0);
        (min_tmp, max_tmp)
    }

    fn calculate_ph_range(morph: &Morphology) -> (f32, f32)
    {
        // What is the starting polymini temperature?
        // TODO: Make this much more data driven
        let mut h_resist = 0.05 *
                           morph.get_traits_of_type(PolyminiTrait::PolyminiSimpleTrait(
                                                      TraitTag::PhBasicResist)).len() as f32;
        let mut c_resist = -0.05 *
                           morph.get_traits_of_type(PolyminiTrait::PolyminiSimpleTrait(
                                                      TraitTag::PhAcidResist)).len() as f32;

        let min_tmp = (BASE_LINE_TMP.0 + c_resist).max(0.0);
        let max_tmp = (BASE_LINE_TMP.1 + h_resist).min(1.0);
        (min_tmp, max_tmp)
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
    thermo: Thermo,
    ph: Ph,
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

        let temp_range = Stats::calculate_temperature_range(&morphology);
        let ph_range = Stats::calculate_ph_range(&morphology);

        Polymini { uuid: uuid,
                   dead: false,
                   morph: morphology,
                   control: control,
                   physics: Physics::new_with_corner(uuid, dim, pos.0, pos.1, 0, corner),
                   thermo: Thermo::new(uuid, temp_range.0, temp_range.1),
                   ph: Ph::new(uuid, ph_range.0, ph_range.1),
                   stats: stats,
                   fitness_statistics: vec![],
                   raw_score: 0.0,
                   species_weighted_fitness: 0.0 }

    }

    pub fn new_from_json(json:&Json, tt: &TranslationTable) -> Option<Polymini>
    {
        match *json 
        {
            Json::Object(ref json_obj) =>
            {

                let morph =  Morphology::new_from_json(&json_obj.get("Morphology").unwrap(), tt).unwrap();
                let control = Control::new_from_json(&json_obj.get("Control").unwrap(), morph.get_sensor_list(),
                                                     morph.get_actuator_list()).unwrap();
                Some(Polymini::new_with_control((0.0,0.0), morph, control))
            },
            _ =>
            {
                None
            }
        }
    }

    pub fn is_alive(&self) -> bool
    {
        !self.is_dead()
    }

    pub fn is_dead(&self) -> bool
    {
        self.dead
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
    pub fn act_phase(&mut self, substep: usize, phys_world: &mut PhysicsWorld, thermo_world: &mut ThermoWorld, ph_world: &mut PhWorld)
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

        self.thermo.act_on(self.physics.get_pos(), &actions, thermo_world);
        self.ph.act_on(self.physics.get_pos(), &actions, ph_world);
    }
    
    pub fn restart(&mut self, random_ctx: &mut PolyminiRandomCtx, placement_func: &PlacementFunction)
    {
        info!("Restarting {} - Had Fitness {}", self.uuid, self.fitness());
        self.physics.reset(random_ctx, placement_func);
        self.fitness_statistics.push(FitnessStatistic::DistanceTravelled(self.physics.get_distance_moved() as u32));

        let norm_pos = self.physics.get_normalized_pos();
        self.fitness_statistics.push(FitnessStatistic::FinalPosition((255.0*norm_pos.0) as u8,
                                                                     (255.0*norm_pos.1) as u8));
    }

    pub fn reset(&mut self, random_ctx: &mut PolyminiRandomCtx, placement_func: &PlacementFunction)
    {
        info!("Reseting {} - Had Fitness {}", self.uuid, self.fitness());
        self.restart(random_ctx, placement_func);
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

    pub fn consequence(&mut self, pworld: &PhysicsWorld, tworld: &ThermoWorld, ph: &PhWorld, substep: usize)
    {
        if self.dead
        {
            return
        }

        self.physics.update_state(pworld);
        let pos_visited = self.physics.get_pos();
        self.fitness_statistics.push(FitnessStatistic::PositionVisited((pos_visited.0 as u32,
                                                                        pos_visited.1 as u32)));
        // Record Move Action
        if (self.physics.get_move_succeded() && self.physics.get_acted())
        {
            self.fitness_statistics.push(FitnessStatistic::Moved);
        }

        self.thermo.update_state(tworld);
        //self.ph.update_state(tworld);
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

    pub fn get_ph(&self) -> &Ph
    {
        &self.ph
    }

    pub fn get_ph_mut(&mut self) -> &mut Ph
    {
        &mut self.ph
    }

    pub fn get_thermo(&self) -> &Thermo
    {
        &self.thermo
    }

    pub fn get_thermo_mut(&mut self) -> &mut Thermo
    {
        &mut self.thermo
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
        if !ctx.has_flag(PolyminiSerializationFlags::PM_SF_DB)
        {
            json_obj.insert("ID".to_owned(), self.get_id().to_json());
        }

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATIC)
        {
            json_obj.insert("Morphology".to_owned(), self.get_morphology().serialize(ctx));
        }


        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_DYNAMIC)
        {
            json_obj.insert("Alive".to_owned(), Json::Boolean(!self.dead));
        }

        json_obj.insert("Control".to_owned(), self.get_control().serialize(ctx));

        if !ctx.has_flag(PolyminiSerializationFlags::PM_SF_DB)
        {
            json_obj.insert("Physics".to_owned(), self.get_physics().serialize(ctx));
            json_obj.insert("Thermo".to_owned(),  self.get_thermo().serialize(ctx));
            json_obj.insert("Ph".to_owned(),      self.get_ph().serialize(ctx));
        }

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATS) || 
           ctx.has_flag(PolyminiSerializationFlags::PM_SF_DB) 
        {
            json_obj.insert("Fitness".to_owned(), self.fitness().to_json());
            json_obj.insert("Raw".to_owned(), self.raw().to_json());
        }
        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATS)
        {
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
