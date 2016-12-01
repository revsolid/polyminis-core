use ::control::*;
use ::environment::*;
use ::evaluation::*;
use ::genetics::*;
use ::morphology::*;
use ::polymini::*;
use ::serialization::*;
use ::uuid::*;

pub struct Species
{
    name: String,
    ga: PolyminiGeneticAlgorithm<Polymini>,
    creation_context: PolyminiCreationCtx,
}
impl Species
{
    pub fn new(pop: Vec<Polymini>) -> Species
    {
        let id = PolyminiUUIDCtx::next();
        let sp_name = format!("Species {}", id);

        // TODO: This configuration should come from somewhere 
        let cfg = PGAConfig { max_generations: 100, population_size: pop.len() as u32,
                              percentage_elitism: 0.2, fitness_evaluators: vec![],
                              percentage_mutation: 0.1, genome_size: 8 };

        //

        //
        Species {
                  name: sp_name,
                  ga: PolyminiGeneticAlgorithm::new(pop, id, cfg),
                  creation_context: PolyminiCreationCtx::empty()
                }
    }

    pub fn new_from(name: String,
                    translation_table: TranslationTable,
                    default_sensors: &Vec<Sensor>, pgaconfig: PGAConfig) -> Species
    {

        let mut inds = vec![];
        let uuid = PolyminiUUIDCtx::next(); 
        let mut ctx = PolyminiRandomCtx::from_seed([0, 1, 2, uuid as u32], format!("Species {}", uuid));

        for i in 0..pgaconfig.population_size
        {
            let morph = Morphology::new_random(&translation_table,
                                               &mut ctx, pgaconfig.genome_size);
            let pos = (ctx.gen_range(0, 100) as f32, ctx.gen_range(0, 100) as f32);

            let mut sensor_list = default_sensors.clone();
            sensor_list.append(&mut morph.get_sensor_list());

            let hl_size = ctx.gen_range(3, 7);

            let control = Control::new_from_random_ctx(sensor_list, morph.get_actuator_list(), hl_size, &mut ctx);

            inds.push(Polymini::new_with_control(pos, morph, control));
        }

        Species {
                  name: name,
                  ga: PolyminiGeneticAlgorithm::new_with(inds, pgaconfig),
                  creation_context: PolyminiCreationCtx::new_from(translation_table, default_sensors.clone(), ctx)
                }
    }

    pub fn reset(&mut self)
    {
        for i in 0..self.ga.get_population().size()
        {
            self.ga.get_population_mut().get_individual_mut(i).reset(&mut self.creation_context.get_random_ctx());
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
        self.ga.evaluate_population();
    }

    pub fn advance_epoch(&mut self)
    {
        self.ga.step(&mut self.creation_context);
        self.reset();
    }

    pub fn get_accum_score(&self) -> f32
    {
        //NOTE: Accumulated Score = the raw scores of all the Polyminis from
        // this species
        self.ga.get_population().iter().fold( 0.0, |mut accum, ind |
        {
            accum += ind.raw();
            accum
        })
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
        json_obj.insert("Name".to_string(), self.name.to_json());

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATIC)
        {
            //Translation Table
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
