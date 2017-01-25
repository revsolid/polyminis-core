use std::collections::{HashSet, HashMap};
use std::cmp::{min, max};
use std::fmt;

use ::actuators::*;
use ::genetics::*;
use ::serialization::*;
use ::sensors::*;
use ::traits::*;
use ::types::*;

//
//
pub type Chromosome = [u8; 4];

//
pub struct PolyminiCreationCtx
{
    pub trans_table: TranslationTable,
    pub default_sensors: Vec<Sensor>,
    pub random_context: PolyminiRandomCtx,
}
impl PolyminiCreationCtx
{
    pub fn empty() -> PolyminiCreationCtx
    {
        PolyminiCreationCtx::new_from(TranslationTable::new(), vec![], PolyminiRandomCtx::new_unseeded("Temporary".to_owned()))
    }
    pub fn new_from(tt: TranslationTable, default_sensors: Vec<Sensor>,
                    rand_ctx: PolyminiRandomCtx) -> PolyminiCreationCtx
    {
        PolyminiCreationCtx { trans_table: tt, default_sensors: default_sensors, random_context: rand_ctx }
    }
}
impl GAContext for PolyminiCreationCtx
{
    fn get_random_ctx(&mut self) -> &mut PolyminiRandomCtx
    {
        &mut self.random_context
    }
}

//
//
#[derive(Debug)]
pub struct AdjacencyInfo
{
    adj: Vec<Direction>
}
impl AdjacencyInfo
{
    fn new(adjacency_info: Vec<Direction>) -> AdjacencyInfo
    {
        AdjacencyInfo { adj: adjacency_info }
    }

    fn get_neighbours(&self, coord: Coord) -> Vec<Coord>
    {
        let mut to_ret = vec![];
        for d in &self.adj
        {
            match *d
            {
                Direction::UP    => 
                {   // UP is special cased so we can add the head correctly
                     // to accomodate art's request to have a 'head'
                    if coord.1 > 0
                    {
                        to_ret.push((coord.0, coord.1 - 1))
                    }
                },
                Direction::DOWN  => to_ret.push( (coord.0,     coord.1 + 1)),
                Direction::LEFT  => to_ret.push( (coord.0 - 1, coord.1)),
                Direction::RIGHT => to_ret.push( (coord.0 + 1, coord.1)),
                _ => panic!("Found incorrect Direction {:?} in adjacency info", d),
            }
        }
        to_ret
    }
}


//
//
// This might be a confusing name, but it refers to a cell in a grid, not an
// actual biological cell
pub struct Cell 
{
    adjacency_info : AdjacencyInfo,
    pm_trait: Trait 
}
impl Cell
{
    fn new(adjacency_info: AdjacencyInfo, pm_trait: Trait) -> Cell
    {
        Cell { adjacency_info: adjacency_info, pm_trait: pm_trait}
    }
}
impl fmt::Debug for Cell
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "{:02X}", self.pm_trait.trait_number)
    }
}
impl Serializable for Cell
{

    fn serialize(&self, ctx: &mut SerializationCtx) -> Json
    {
        self.pm_trait.serialize(ctx)
    }
}


//
//
// A design note on Translation Table:
// The translation table is basically a function F(t, n) -> T
// t being a Tier and N being a number. T being a full on trait.
//
// 
pub type  TTKey = (TraitTier, u8);
impl Serializable for TTKey
{
    fn serialize(&self, ctx:&mut SerializationCtx) -> Json
    {
        let (t, n) = *self;
        let mut json_obj = pmJsonObject::new();
        json_obj.insert("Tier".to_owned(), t.serialize(ctx));
        json_obj.insert("Number".to_owned(), n.to_json());
        Json::Object(json_obj)
    }
}
impl Deserializable for TTKey
{
    fn new_from_json(json: &Json, _: &mut SerializationCtx) -> Option<TTKey>
    {
        match *json
        {
            Json::Object(ref json_obj) =>
            {
                let trait_tier = TraitTier::from(json_obj.get("Tier").unwrap().as_u64().unwrap() as u8);
                let num = json_obj.get("Number").unwrap().as_u64().unwrap() as u8;
                Some((trait_tier, num))
            },
            _ => 
            {
                error!("Wrong Json type for TTKey Desrialization");
                None
            }
        }
    }
}

pub struct TranslationTable
{
    trait_table:  HashMap<TTKey, PolyminiTrait>,
}
impl TranslationTable
{
    pub fn new() -> TranslationTable
    {
        TranslationTable::new_from(&HashMap::new(), &HashSet::new())
    }

    pub fn new_from(trait_table: &HashMap<TTKey, PolyminiTrait>, 
                    active:  &HashSet<TTKey>) -> TranslationTable
    {
        // Copy over the active mappings only
        let mut filtered_table = HashMap::new();
        for (k,v) in trait_table
        {
            if active.contains(k)
            {
                filtered_table.insert(*k, *v);
            }
        }
        TranslationTable { trait_table: filtered_table }
    }

    pub fn new_from_json(json: &Json, trait_table: &HashMap<TTKey, PolyminiTrait>) -> Option<TranslationTable>
    {
        match *json
        {
            Json::Array(ref json_arr) =>
            {
                let mut active = HashSet::new();
                for elem in json_arr
                {
                    let tt_key = TTKey::new_from_json(elem, &mut SerializationCtx::new()).unwrap();
                    active.insert(tt_key);
                }
                Some(TranslationTable::new_from(trait_table, &active))
            },
            _ =>
            {
                error!("Translation Table fed wrong Json - {}", json.to_string());
                None
            }
        }
    }
    
    fn create_for_chromosome(&self,
                             chromosome: Chromosome) -> Cell
    {

        //TODO: Control / Metadata Payload
        let _ = 0xFF & chromosome[0];
        let ap = 0xFF & chromosome[1];
        let gp1 = ((0xFF & chromosome[2]) as u16) << 8;
        let gp  = gp1 + (0xFF & chromosome[3]) as u16;

        let dirs = vec![Direction::UP, Direction::DOWN, Direction::LEFT, Direction::RIGHT];
        let mut adj_dirs = vec![];

        for i in 0..4
        {
            if 1<<i & ap != 0 
            {
                adj_dirs.push(dirs[i]);
            }
        }

        //TODO: Make this configurable - Using a list of Transform + Chain could work
        //TIER I
        let mut tier: u8 = 1;
        let mut trait_num = ( ( gp & (0xFF<<8) ) >> 8 ) as u8;

        if trait_num == TIER_ONE_TO_TWO_CHAIN
        {
            //TIER II
            tier += 1;
            trait_num = ((gp & (0xF<<4)) >> 4) as u8;
            if trait_num == TIER_TWO_TO_THREE_CHAIN
            {
                //TIER III
                tier += 1;
                trait_num = (gp & 0xF) as u8;
            }
        }

        // TODO:
        // Get PolyminiTrait
        let trait_tier = TraitTier::from(tier);
        let mut polymini_trait;
        match self.trait_table.get(&(trait_tier, trait_num))
        {
            Some(trait_value) =>
            {
                polymini_trait = *trait_value;
                debug!("Morphology::ChromosomeCreate {:?}", polymini_trait);
            }
            None =>
            { 
               polymini_trait = PolyminiTrait::PolyminiSimpleTrait(TraitTag::Empty);
            }
        }

        Cell::new(AdjacencyInfo::new(adj_dirs),
                  Trait::new(TraitTier::from(tier), trait_num, polymini_trait))
    }
}
impl Serializable for TranslationTable
{
    fn serialize(&self, ctx: &mut SerializationCtx) -> Json
    {
        let mut json_arr = pmJsonArray::new();

        for (k, v) in &self.trait_table
        {
            json_arr.push(k.serialize(ctx));
        }

        Json::Array(json_arr)
    }
}
//
// Positions - Maps a Coordinate to a position in the Cell list
const TOTAL_ORIENTATIONS: usize = 4; 
struct Representation
{
    cells: Vec<Cell>,
    positions: [HashMap<Coord, usize>; TOTAL_ORIENTATIONS],
    dimensions: (u8, u8),
    corners: [(i8, i8); TOTAL_ORIENTATIONS],
}
impl Representation
{
    pub fn new(positions: Vec<Coord>, cells: Vec<Cell>,
               dimensions: (u8, u8), corner: (i8, i8)) -> Representation
    {
        let mut all_positions : [HashMap<Coord, usize>; TOTAL_ORIENTATIONS] = [HashMap::new(), HashMap::new(),
                                                                               HashMap::new(), HashMap::new()];

        // TODO: This should be configurable but enforceable
        assert_eq!(corner.1, 0);

        for i in 0..positions.len()
        {
            all_positions[0].insert(positions[i], i);
        }

        all_positions[1] = Representation::rotate(&all_positions[0]); 
        all_positions[2] = Representation::rotate(&all_positions[1]); 
        all_positions[3] = Representation::rotate(&all_positions[2]); 

        let mut corners = [(0,0); TOTAL_ORIENTATIONS];

        let minx = corner.0;
        let miny = corner.1;
        let maxx = dimensions.0 as i8 + minx - 1;
        let maxy = dimensions.1 as i8 + miny - 1;

        corners[0] = (   minx,       miny);
        corners[1] = (   miny,    -1*maxx);
        corners[2] = (-1*maxx,    -1*maxy);
        corners[3] = (-1*maxy,       minx);

        Representation { cells: cells, positions: all_positions, dimensions: dimensions,
                         corners: corners }
    }

    fn rotate(positions: &HashMap<Coord, usize>) -> HashMap<Coord, usize>
    {
        let mut to_ret: HashMap<Coord, usize> = HashMap::new();
        for (c, p) in positions.iter()
        {
            // Rotation matrix [ cosA  -sinA ]
            //                 [ sinA   cosA ]
            // A = 90 degrees, sinA = 1, cosA = 0
            let new_c = (c.1, -1 * c.0);
            to_ret.insert(new_c, *p);
        }
        to_ret
    }
}
impl Serializable for Representation
{
    fn serialize(&self, ctx: &mut SerializationCtx) -> Json
    {
        let mut json_arr = pmJsonArray::new();
        for (coord, inx) in &self.positions[0]
        {
            //json_obj.insert(format!("{:?}", coord), self.cells[*inx].serialize(ctx));
            let mut json_obj = pmJsonObject::new();
            {
                let mut coord_json = pmJsonObject::new();
                coord_json.insert("x".to_owned(), coord.0.to_json());
                coord_json.insert("y".to_owned(), coord.1.to_json());
                json_obj.insert("Coord".to_owned(), Json::Object(coord_json));
            }
            json_obj.insert("Trait".to_owned(), self.cells[*inx].serialize(ctx));
            json_arr.push(Json::Object(json_obj));
        }
        Json::Array(json_arr)
    }
}
impl fmt::Debug for Representation 
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        let mut result = write!(f, "\n");
        let dim = [self.dimensions.0, self.dimensions.1];
        for p in 0..4
        {
            // Flip-flop width and height for rotations
            let w = dim[p%2];
            let h = dim[(p+1)%2];

            let _ = write!(f, "\n");
            for i in 0..h 
            {
                for j in 0..w 
                {
                    let mut trans_coord = (j as i8, i as i8);
                    trans_coord.0 += self.corners[p].0;
                    trans_coord.1 += self.corners[p].1;

                    match self.positions[p].get(&trans_coord)
                    {
                        Some(x) => {
                            let _ = write!(f, "{:?}", self.cells[*x]);
                        },
                        None => {
                            let _ = write!(f, "..");
                        }
                    }
                }
                result = write!(f, "\n");
            }
        }
        result
    }
}

//
//
#[derive(Debug)]
pub struct Morphology
{
    dimensions: (u8, u8),
    representations: Representation,
    original_chromosome: Vec<Chromosome>
}
impl Morphology
{
    pub fn empty() -> Morphology
    {
        Morphology { dimensions: (0,0),
                     representations: Representation::new(vec![], vec![], (0,0), (0,0)),
                     original_chromosome: vec![]
        }
    }

    pub fn new(chromosomes: &Vec<Chromosome>,
               translation_table: &TranslationTable) -> Morphology
    {
        let mut original_chromosome = vec![];
        for c in chromosomes
        {
            original_chromosome.push(*c);
        }

        let rep = Morphology::create_representation(chromosomes, translation_table);
        
        Morphology { dimensions: rep.dimensions, representations: rep, original_chromosome: original_chromosome }
    }

    pub fn new_from_json(json: &Json, tt: &TranslationTable) -> Option<Morphology>
    {
        match *json
        {
            Json::Object(ref json_obj) =>
            {
                let mut o_chromosome = vec![];
                
                let mut arr = json_obj.get("Chromosome").unwrap().as_array().unwrap();
                let mut iter = arr.iter();
                for e in arr
                {
                    let block = e.as_array().unwrap();
                    o_chromosome.push([
                        block[0].as_u64().unwrap() as u8,
                        block[1].as_u64().unwrap() as u8,
                        block[2].as_u64().unwrap() as u8,
                        block[3].as_u64().unwrap() as u8,
                    ]);
                }
                Some(Morphology::new(&o_chromosome, tt))
            },
            _ =>
            {
                None
            }
        }
    }

    pub fn new_random(translation_table: &TranslationTable, random_ctx: &mut PolyminiRandomCtx, genome_size: usize) -> Morphology
    {
        let mut random_chromosomes = vec![];

        for i in 0..genome_size
        {
            random_chromosomes.push([ random_ctx.gen::<u8>(),
                                      random_ctx.gen::<u8>(),
                                      random_ctx.gen::<u8>(),
                                      random_ctx.gen::<u8>() ]);
        }

        Morphology::new(&random_chromosomes, translation_table)
    }

    fn create_representation (chromosomes: &Vec<Chromosome>, translation_table: &TranslationTable) -> Representation
    {
        let mut cells = vec![];
        for c in chromosomes
        {
            cells.push(translation_table.create_for_chromosome(*c));
        }

        let mut visited: HashSet<Coord> = HashSet::new();
        let mut stack: Vec<Coord> = Vec::new();
        let mut positions: Vec<Coord> = Vec::new();

        // Iterate through the cells gathering adjacency info
        let mut curr_coord = (0, 0);
        // Init all of these to 0, as we are 100% to have the nucleus at 0,0
        let mut minx = 0;
        let mut maxx = 0;
        let mut miny = 0;
        let mut maxy = 0;


        positions.push(curr_coord);
        for cell in &mut cells
        {
            let mut coords = cell.adjacency_info.get_neighbours(curr_coord);
            for coord in &mut coords
            {
                if visited.contains(coord)
                {
                    continue;
                }
                stack.push(*coord);
                positions.push(*coord);

                visited.insert(*coord);
            }
            match stack.pop()
            {
                Some(c) => { curr_coord = c; },
                None => { break; }
            }
        }

        let l = min(cells.len(), positions.len());
        let drain_cell: Vec<Cell> = cells.drain(0..l).collect();
        let drain_pos: Vec<Coord> = positions.drain(0..l).collect();



        for p in &drain_pos
        {
            minx = min(minx, p.0);
            miny = min(miny, p.1);

            maxx = max(maxx, p.0);
            maxy = max(maxy, p.1);
        }

        let w = (maxx - minx + 1) as u8;
        let h = (maxy - miny + 1) as u8;

        Representation::new(drain_pos, drain_cell, (w, h), (minx, miny))
    }

    pub fn get_dimensions(&self) -> (u8, u8)
    {
        return self.dimensions
    }

    pub fn crossover(&self, other: &Morphology, creation_ctx: &mut PolyminiCreationCtx) -> Morphology
    {
        let mut chromosomes = vec![];

        // TODO: A LOT of magic numbers :S
        // bit to make the cut at
        let cross_point_chromosome = creation_ctx.get_random_ctx().gen_range(0, self.original_chromosome.len());
        let cross_point_allele = creation_ctx.get_random_ctx().gen_range(0, 4);
        let cross_point_bit = creation_ctx.get_random_ctx().gen_range(0, 8);


        for i in 0..cross_point_chromosome
        {
            chromosomes.push([ self.original_chromosome[i][0],
                               self.original_chromosome[i][1],
                               self.original_chromosome[i][2],
                               self.original_chromosome[i][3] ])
        }

        let mut link_chromosome = [0; 4];

        for lc in 0..cross_point_allele
        {
            link_chromosome[lc] = self.original_chromosome[cross_point_chromosome][lc];
        }

        let link_byte;

        // make the mask u16 to allow space for overflowing
        let mut mask : u16 = ( 1 << (cross_point_bit+1)) - 1;
        mask = mask << (8 - cross_point_bit);

        let mask_2 : u16 = (1 << ((8 - cross_point_bit))) - 1;

        let cross_point_chromosome_2 = creation_ctx.get_random_ctx().gen_range(0, other.original_chromosome.len()); 

        for lc in cross_point_allele..4
        {
            link_chromosome[lc] = other.original_chromosome[cross_point_chromosome_2][lc];
        }

        debug!("{:X} {:X}", mask as u8, mask_2 as u8);
        link_byte = ((mask as u8) & (self.original_chromosome[cross_point_chromosome][cross_point_allele])) +
                    ((mask_2 as u8) & (other.original_chromosome[cross_point_chromosome_2][cross_point_allele]));
        link_chromosome[cross_point_allele] = link_byte;
        chromosomes.push(link_chromosome);

        for j in cross_point_chromosome_2 + 1..other.original_chromosome.len()
        {
            chromosomes.push([ other.original_chromosome[j][0],
                               other.original_chromosome[j][1],
                               other.original_chromosome[j][2],
                               other.original_chromosome[j][3] ])
        }

        debug!("{} {} {} {}", cross_point_chromosome, cross_point_allele, cross_point_bit, cross_point_chromosome_2);
        debug!("{:?} {:?} {:?}", link_chromosome, self.original_chromosome[cross_point_chromosome],
                 other.original_chromosome[cross_point_chromosome_2]);
        debug!("{}", link_byte);

        Morphology::new(&chromosomes, &creation_ctx.trans_table)
    }

    pub fn mutate(&mut self, random_ctx: &mut PolyminiRandomCtx, table: &TranslationTable)
    {
        for i in 0..random_ctx.gen_range(1, 8)
        {
            let chromosome_to_mutate = random_ctx.gen_range(0, self.original_chromosome.len());
            let allele_to_mutate = random_ctx.gen_range(0, 4);
            self.original_chromosome[chromosome_to_mutate][allele_to_mutate] = random_ctx.gen::<u8>();
        }

        // After mutating the chromosome is possible the whole morphology representation has
        // changed
        self.representations = Morphology::create_representation(&self.original_chromosome, table);
        self.dimensions = self.representations.dimensions;
    }


    pub fn get_actuator_list(&self) -> Vec<Actuator>
    {

        let mut actuators = vec![];

        for (k, v) in &self.representations.positions[0]
        {
            let c = &self.representations.cells[*v];
            match c.pm_trait.pm_trait 
            {
                PolyminiTrait::PolyminiActuator(t) =>
                {
                    actuators.push(Actuator::new(t, 0,  *k));
                },
                _ =>
                {
                }
            }
        }

        actuators
    }

    pub fn get_sensor_list(&self) -> Vec<Sensor>
    {
        let mut sensors = vec![];
        for (i, c) in self.representations.cells.iter().enumerate()
        {
            match c.pm_trait.pm_trait 
            {
                PolyminiTrait::PolyminiSensor(t) =>
                {
                    sensors.push(Sensor::new(t, i));
                },
                _ =>
                {
                }
            }
        }
        sensors
    }

    pub fn get_traits_of_type(&self, trait_type: PolyminiTrait) -> Vec<PolyminiTrait>
    {
        let mut to_ret = vec![];
        self.representations.cells.iter().fold(&mut to_ret, |accum, cell|
        {
            debug!("Trait Type {:?}", trait_type);
            debug!("Trait Type (cell) {:?}", cell.pm_trait.pm_trait);
            if cell.pm_trait.pm_trait == trait_type
            {
                accum.push(cell.pm_trait.pm_trait);
            }
            accum
        });
        to_ret
    }

    pub fn get_total_cells(&self) -> usize
    {
        self.representations.cells.len()
    }

    pub fn get_corner(&self) -> (i8, i8)
    {
        self.representations.corners[0]
    }
}
impl Serializable for Morphology
{
    fn serialize(&self,  ctx: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();
        
        json_obj.insert("Body".to_string(), self.representations.serialize(ctx));


        let mut json_arr = pmJsonArray::new();
        for c in &self.original_chromosome
        {
            json_arr.push(c.to_json());
        }
        json_obj.insert("Chromosome".to_string(), Json::Array(json_arr));

        Json::Object(json_obj)
    }
}

//
//
#[cfg(test)]
mod test
{
    use ::genetics::*;
    use ::morphology::*;
    use ::serialization::*;
    use ::types::*;
    #[test]
    fn test_adjacency_vertical()
    {
        let adj_info = AdjacencyInfo{ adj: vec![Direction::UP, Direction::DOWN] };
        let neighbours = adj_info.get_neighbours((0, 1));
        assert_eq!(neighbours.len(), 2);
        assert_eq!( (neighbours[0].0, neighbours[0].1), (0,0));
        assert_eq!( (neighbours[1].0, neighbours[1].1), (0,2));
    }

    #[test]
    fn test_adjacency_vertical_2()
    {
        let adj_info = AdjacencyInfo{ adj: vec![Direction::UP, Direction::DOWN] };
        let neighbours = adj_info.get_neighbours((0, 0));
        assert_eq!(neighbours.len(), 1);
        assert_eq!((neighbours[0].0, neighbours[0].1), (0,1));
    }

    #[test]
    fn test_chromosomes_to_morphology()
    {
        // UP & RIGHT, up will be ignored
        let v1: u8 = 0x09;
        // UP, RIGHT & DOWN, up will be ignored
        let v2: u8 = 0x0B;
        // Expected shape:
        // XXX
        // .X.
        // Y = Head, X = Cell, . = Empty 
        let chromosomes = vec![[0, v1, 0x6A, 0xAD],
                               [0, v2, 0xBE, 0xDA],
                               [0,  0, 0xBE, 0xEF],
                               [0,  0, 0xDB, 0xAD]];

        let morph = Morphology::new(&chromosomes, &TranslationTable::new());

        assert_eq!((3, 2), (morph.dimensions.0, morph.dimensions.1));
        assert_eq!(morph.original_chromosome[0][1], v1);
        assert_eq!(morph.original_chromosome[1][1], v2);
        debug!("{:?}", morph);
    }

    #[test]
    fn test_chromosomes_to_morphology_2()
    {
        // LEFT & RIGHT
        let v1: u8 = 0x0C;
        let chromosomes = vec![[0, v1, 0x6A, 0xAD],
                               [0,  0, 0xBE, 0xDA],
                               [0,  0, 0xBE, 0xEF],
                               [0,  0, 0xDB, 0xAD]];

        let morph = Morphology::new(&chromosomes, &TranslationTable::new());
        assert_eq!((3, 1), (morph.dimensions.0, morph.dimensions.1));
        assert_eq!(morph.original_chromosome[0][1], v1);
        debug!("{:?}", morph);
    }

    #[test]
    fn test_morphology_crossover()
    {
        let c1 = vec![[0, 0x0C, 0x6A, 0xAD],
                      [0,    0, 0xBE, 0xDA],
                      [0,    0, 0xBE, 0xEF],
                      [0,    0, 0xDB, 0xAD]];
        let c2 = vec![[0, 0x0C, 0x6A, 0xAD],
                      [0,    0, 0xBE, 0xDA],
                      [0,    0, 0xBE, 0xEF],
                      [0,    0, 0xDB, 0xAD]];

        let morph = Morphology::new(&c1, &TranslationTable::new());
        let morph_2 = Morphology::new(&c2, &TranslationTable::new());

        let child = morph.crossover(&morph_2, &mut PolyminiCreationCtx::empty()); 

        debug!("{:?}", child);
    }

    #[test]
    fn test_morphology_crossover_2()
    {
        let c1 = vec![[0, 0x09, 0x6A, 0xAD],
                      [0, 0x0B, 0xFF, 0xFF],
                      [0,    0, 0xFF, 0xFF],
                      [0,    0, 0xFF, 0xFF]];
        let c2 = vec![[0, 0x0C, 0x00, 0x00],
                      [0,    0, 0x00, 0x00],
                      [0,    0, 0x00, 0x00],
                      [0,    0, 0x00, 0x00]];

        let morph = Morphology::new(&c1, &TranslationTable::new());
        let morph_2 = Morphology::new(&c2, &TranslationTable::new());

        let child = morph.crossover(&morph_2, &mut PolyminiCreationCtx::empty()); 

        debug!("{:?}", child);
    }

    #[test]
    fn test_morphology_mutate()
    {
        let c1 = vec![[0, 0x09, 0x6A, 0xAD],
                      [0, 0x0B, 0xFF, 0xFF],
                      [0,    0, 0xFF, 0xFF],
                      [0,    0, 0xFF, 0xFF]];
        let c2 = vec![[0, 0x0C, 0x00, 0x00],
                      [0,    0, 0x00, 0x00],
                      [0,    0, 0x00, 0x00],
                      [0,    0, 0x00, 0x00]];

        let mut morph = Morphology::new(&c1, &TranslationTable::new());
        debug!("{:?}", morph);
        morph.mutate(&mut PolyminiRandomCtx::from_seed([5,7,8,9], "Test Mutate".to_owned()), &TranslationTable::new());
        debug!("{:?}", morph);
    }

    #[test]
    fn test_morphology_serialization()
    {
        let mut morph = Morphology::new_random(&TranslationTable::new(), &mut PolyminiRandomCtx::new_unseeded("Test Serialization".to_owned()),8);
        let json_1 = morph.serialize(&mut SerializationCtx::new());
        let morph_2 = Morphology::new_from_json(&json_1, &TranslationTable::new()).unwrap();
        let json_2 = morph_2.serialize(&mut SerializationCtx::new());

        assert_eq!(json_1.to_string(), json_2.to_string());
    }
}
