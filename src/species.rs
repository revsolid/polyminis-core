use ::control::*;
use ::environment::*;
use ::evaluation::*;
use ::genetics::*;
use ::instincts::*;
use ::morphology::*;
use ::polymini::*;
use ::physics::*;
use ::serialization::*;
use ::uuid::*;
use ::traits::*;

use std::collections::HashMap;
use std::hash::{ Hash, Hasher, SipHasher };

pub type IndividualFilterFunction = Fn(&pmJsonArray, &TranslationTable, &Vec<Sensor>) -> Vec<Polymini>;

pub struct SpeciesStats
{
}
impl SpeciesStats
{
}
impl Serializable for SpeciesStats
{
    fn serialize(&self,  ctx: &mut SerializationCtx) -> Json
    {
        Json::Null
    }
}


pub struct Species
{
    name: String,
    ga: PolyminiGeneticAlgorithm<Polymini>,
    creation_context: PolyminiCreationCtx,
    placement_function: Box<PlacementFunction>,
    accumulated_score: f32,
    percentage_population: f32,
    instinct_weights: HashMap<Instinct, f32>,
    stats: SpeciesStats,
}
impl Species
{
    pub fn new(pop: Vec<Polymini>) -> Species
    {
        let id = PolyminiUUIDCtx::next();
        let sp_name = format!("Species {}", id);

        // Default configuration
        let cfg = PGAConfig { population_size: pop.len() as u32,
                              percentage_elitism: 0.2, fitness_evaluators: vec![], accumulates_over: false,
                              percentage_mutation: 0.1, genome_size: 8 };// instinct_weights: HashMap::new() };

        //
        Species {
                  name: sp_name,
                  ga: PolyminiGeneticAlgorithm::new(pop, id, cfg),
                  creation_context: PolyminiCreationCtx::empty(),
                  placement_function: Box::new( |ctx: &mut PolyminiRandomCtx|
                                              {
                                                  ( (ctx.gen_range(0.0, 100.0) as f32).floor(),
                                                    (ctx.gen_range(0.0, 100.0) as f32).floor())
                                              }),
                  accumulated_score: 0.0,
                  percentage_population: 0.0,
                  instinct_weights: HashMap::new(),
                  stats: SpeciesStats {}
                }
    }

    pub fn new_from(name: String,
                    translation_table: TranslationTable,
                    default_sensors: &Vec<Sensor>, pgaconfig: PGAConfig,
                    placement_func: Box<PlacementFunction>) -> Species
    {

        let mut inds = vec![];
        let uuid = PolyminiUUIDCtx::next(); 
        let mut s = SipHasher::new();
        let name_hash = name.hash(&mut s);
        let hash_v = s.finish();
        let mut ctx = PolyminiRandomCtx::from_seed([
            ((hash_v >> 32) & 0xFFFFFFFF) as u32,
            (hash_v         & 0xFFFFFFFF) as u32,
            name.len()                    as u32,
            3151,
        ], name.clone());

        for i in 0..pgaconfig.population_size
        {
            let morph = Morphology::new_random(&translation_table,
                                               &mut ctx, pgaconfig.genome_size);
            let pos = placement_func(&mut ctx);

            let mut sensor_list = default_sensors.clone();
            sensor_list.append(&mut morph.get_sensor_list());

            let hl_size = ctx.gen_range(3, 7);

            let control = Control::new_from_random_ctx(sensor_list, morph.get_actuator_list(), hl_size, &mut ctx);

            inds.push(Polymini::new_with_control(pos, morph, control));
        }

        Species {
                  name: name,
                  ga: PolyminiGeneticAlgorithm::new_with(inds, pgaconfig),
                  creation_context: PolyminiCreationCtx::new_from(translation_table, default_sensors.clone(), ctx),
                  placement_function: placement_func,
                  accumulated_score: 0.0,
                  percentage_population: 0.0,
                  instinct_weights: HashMap::new(),
                  stats: SpeciesStats{},
                }
    }

    pub fn new_from_json(json: &Json, default_sensors: &Vec<Sensor>,
                         placement_func: Box<PlacementFunction>,
                         master_table: &HashMap<(TraitTier, u8), PolyminiTrait>,
                         filter_function: Option<Box<IndividualFilterFunction>>) -> Option<Species>
    {
        match *json
        {
            Json::Object(ref json_obj) => 
            {
                if !JsonUtils::verify_has_fields(&json_obj, &vec!["TranslationTable".to_owned(),
                                                                  "GAConfiguration".to_owned(),
                                                                  "InstinctWeights".to_owned()])
                {
                    error!("Verify Fields Failed");
                    return None
                }

                let translation_table = TranslationTable::new_from_json(json_obj.get("TranslationTable").unwrap(), master_table).unwrap();
                let pgaconfig = match PGAConfig::new_from_json(json_obj.get("GAConfiguration").unwrap(), &mut SerializationCtx::new())
                {

                    Some(config_v) =>
                    {
                        config_v
                    },
                    None =>
                    {
                        PGAConfig::defaults()
                    }
                };

                let mut iw = HashMap::new();

                let name = json_obj.get("SpeciesName").unwrap_or(&Json::Null).as_string().unwrap_or("Test Species").clone().to_string();
                let percentage = json_obj.get("Percentage").unwrap_or(&Json::Null).as_f64().unwrap_or(0.0) as f32;

                let mut s = SipHasher::new();
                let name_hash = name.hash(&mut s);
                let hash_v = s.finish();
                let mut ctx = PolyminiRandomCtx::from_seed([
                    ((hash_v >> 32) & 0xFFFFFFFF) as u32,
                    (hash_v         & 0xFFFFFFFF) as u32,
                    name.len()                    as u32,
                    3151,
                ], name.clone());

                match json_obj.get("InstinctWeights")
                {
                    Some(&Json::Object(ref json_obj)) =>
                    {
                        for (k,v) in json_obj.iter()
                        {
                            // TODO: This 'to_json' is pretty redundant
                            let i = Instinct::new_from_json(&k.to_json(), &mut SerializationCtx::new()).unwrap();
                            iw.insert(i, v.as_f64().unwrap() as f32);
                        }
                    },
                    _ =>
                    {
                    }
                };

                let empty_arr = vec![];
                let empty_jarr = Json::Array(vec![]);
                let inds_json = json_obj.get("Individuals").unwrap_or(&empty_jarr).as_array().unwrap_or(&empty_arr);
                let inds: Vec<Polymini> = match filter_function
                {
                    None =>
                    {
                        let mut ret = vec![]; 
                        // Default is just add every individual once
                        for ind_json in inds_json
                        {
                            let ind = Polymini::new_from_json(ind_json, &translation_table, default_sensors);
                            match ind 
                            {
                                Some(_) => {},
                                None => { error!("Polyminy couldn't be created from {:?}", ind_json); }
                            }
                            ret.push(ind.unwrap());
                        }
                        ret
                    },
                    Some(filter) =>
                    {
                        filter(inds_json, &translation_table, default_sensors)
                    }
                };
                
                
                if inds.len() == 0
                {
                    Some(Species::new_from(name, translation_table, default_sensors, pgaconfig, placement_func))
                }
                else
                {
                    let mut s = Species { name: name,
                                   ga: PolyminiGeneticAlgorithm::new_with(inds, pgaconfig),
                                   creation_context: PolyminiCreationCtx::new_from(translation_table, default_sensors.clone(), ctx),
                                   placement_function: placement_func,
                                   accumulated_score: 0.0,
                                   percentage_population: percentage,
                                   instinct_weights: iw,
                                   stats: SpeciesStats{}
                                 };
                    s.restart();

                    Some(s)
                }
            },
            _ =>
            {
                error!("Species JSON is not an Object"); 
                None
            }
        }
    }

    pub fn restart(&mut self)
    {
        for i in 0..self.ga.get_population().size()
        {
            self.ga.get_population_mut().get_individual_mut(i).restart(&mut self.creation_context.get_random_ctx(),
                                                                       &(*self.placement_function));
        }
    }

    pub fn reset(&mut self)
    {
        for i in 0..self.ga.get_population().size()
        {
            self.ga.get_population_mut().get_individual_mut(i).reset(&mut self.creation_context.get_random_ctx(),
                                                                     &(*self.placement_function));
        }
    }

    pub fn get_name(&self) -> &String
    {
        &self.name
    }

    pub fn get_generation(&self) -> &PolyminiGeneration<Polymini>
    {
        self.ga.get_population()
    }

    pub fn get_generation_mut(&mut self) -> &mut PolyminiGeneration<Polymini>
    {
        self.ga.get_population_mut()
    }

    pub fn get_best(&self) -> &Polymini
    {
        self.ga.get_population().get_individual(0)
    }

    pub fn evaluate(&mut self)
    {
        self.ga.evaluate_population(&self.instinct_weights);

        let species_score = self.ga.get_population().iter().fold(0.0,
                                |mut accum, ind|
                                {
                                    accum += ind.raw();
                                    accum
                                });

        self.accumulated_score = species_score;
    }

    pub fn advance_epoch(&mut self)
    {
        self.ga.step(&mut self.creation_context);
        self.reset();
    }

    pub fn set_ga_config(&mut self, config: PGAConfig)
    {
        self.ga.change_config(config);
    }

    pub fn get_percentage(&mut self) -> f32
    {
        self.percentage_population
    }

    pub fn set_percentage(&mut self, perc: f32)
    {
        self.percentage_population = perc;
    }

    pub fn get_accum_score(&self) -> f32
    {
        self.accumulated_score
    }

    pub fn dump_random_ctx(&mut self)
    {
        info!("{:?}", self.creation_context.get_random_ctx());
    }
}

impl Serializable for Species
{
    fn serialize(&self,  ctx: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();
        json_obj.insert("SpeciesName".to_string(), self.name.to_json());

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATIC)
        {
            json_obj.insert("TranslationTable".to_string(), self.creation_context.trans_table.serialize(ctx));
            json_obj.insert("GAConfiguration".to_string(), self.ga.get_config().serialize(ctx));
            json_obj.insert("Percentage".to_string(), self.percentage_population.to_json());
        }

        let mut pop_arr = pmJsonArray::new();
        if self.ga.get_population().size() > 0
        {
            for ind in self.ga.get_population().iter()
            {
                pop_arr.push(ind.serialize(ctx));
            }
            json_obj.insert("Individuals".to_string(), Json::Array(pop_arr));
        }
        Json::Object(json_obj)
    }
}
