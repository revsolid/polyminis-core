use ::morphology::*;
use ::polymini::*;
use ::genetics::*;
use ::serialization::*;
use ::uuid::*;

pub struct Species
{
    name: String,
    ga: PolyminiGeneticAlgorithm<Polymini>,
    translation_table: TranslationTable,
}
impl Species
{
    pub fn new(pop: Vec<Polymini>) -> Species
    {
        let id = PolyminiUUIDCtx::next();
        let sp_name = format!("Species {}", id);

        // TODO: Fix this 
        let cfg = PGAConfig { max_generations: 100, population_size: pop.len() as u32,
                              percentage_elitism: 0.2, };

        //

        //
        Species { name: sp_name,
                  ga: PolyminiGeneticAlgorithm::new(pop, id, cfg),
                  translation_table: TranslationTable::new() }
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

    pub fn advance_epoch(&mut self)
    {
        self.ga.step();
    }
}

impl Serializable for Species
{
    fn serialize(&self,  ctx: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();
        json_obj.insert("name".to_string(), self.name.to_json());

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATIC)
        {
            //Translation Table
        }

        let mut pop_arr = pmJsonArray::new();
        for ind in self.ga.get_population().iter()
        {
            pop_arr.push(ind.serialize(ctx));
        }
        json_obj.insert("population".to_string(), Json::Array(pop_arr));
        Json::Object(json_obj)
    }
}
