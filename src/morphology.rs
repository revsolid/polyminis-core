use std::collections::{HashSet, HashMap};
use std::cmp::{min, max};
use std::fmt;

//
//
pub type Coord = (i32, i32);

#[derive(Copy, Clone, Debug)]
pub enum Directions
{
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

//
//
pub struct Gene // ?
{
    control_payload: u8,
    adjacency_payload: u8,
    genetic_payload: u16,
}
impl Gene
{ 
    pub fn new(byte: u32) -> Gene
    {

        let cp = (((0xFF << 24) & byte) >> 24) as u8;
        let ap = (((0xFF << 16) & byte) >> 16) as u8;
        let gp = (0xFFFF & byte) as u16;

        Gene { control_payload: cp,
               adjacency_payload: ap,
               genetic_payload: gp,
             }
    }
}


//
//
#[derive(Debug)]
pub struct AdjacencyInfo
{
    adj: Vec<Directions>
}
impl AdjacencyInfo
{
    fn get_neighbours(&self, coord: Coord) -> Vec<Coord>
    {
        let mut to_ret = vec![];
        for d in &self.adj
        {
            match *d
            {
                Directions::UP    => 
                {   // UP is special cased so we can add the head correctly
                     // to accomodate art's request to have a 'head'
                    if coord.1 > 0
                    {
                        to_ret.push((coord.0, coord.1 - 1))
                    }
                },
                Directions::DOWN  => to_ret.push( (coord.0,     coord.1 + 1)),
                Directions::LEFT  => to_ret.push( (coord.0 - 1, coord.1)),
                Directions::RIGHT => to_ret.push( (coord.0 + 1, coord.1)),
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
}
impl Cell
{
    fn new(adjacency_info: AdjacencyInfo) -> Cell
    {
        Cell { adjacency_info: adjacency_info }
    }
}
impl fmt::Debug for Cell
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "X")
    }
}


//
//
struct Representation
{
    cells: Vec<Cell>,
    positions: [HashMap<Coord, usize>; 4],
    dimensions: (u8, u8),
    corner: (i32, i32),
}
impl Representation
{
    pub fn new(positions: Vec<Coord>, cells: Vec<Cell>,
               dimensions: (u8, u8), corner: (i32, i32)) -> Representation
    {
        let mut all_positions : [HashMap<Coord, usize>; 4] = [HashMap::new(), HashMap::new(),
                                                              HashMap::new(), HashMap::new()];

        for i in 0..positions.len()
        {
            all_positions[0].insert(positions[i], i);
        }

        all_positions[1] = Representation::rotate(&all_positions[0]); 
        all_positions[2] = Representation::rotate(&all_positions[1]); 
        all_positions[3] = Representation::rotate(&all_positions[2]); 

        Representation { cells: cells, positions: all_positions, dimensions: dimensions,
                         corner: corner }
    }

    fn rotate(positions: &HashMap<Coord, usize>) -> HashMap<Coord, usize>
    {
        let mut to_ret: HashMap<Coord, usize> = HashMap::new();
        for (c, p) in positions.iter()
        {
            // Rotation matrix [ cosA  -sinA ]
            //                 [ sinA   cosA  ]
            // A = 90 degrees
            let new_c = (c.1, -1 * c.0);
            to_ret.insert(new_c, *p);
        }
        to_ret
    }
}
impl fmt::Debug for Representation 
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        let mut result = write!(f, "\n");
        for i in 0..self.dimensions.1
        {
            for j in 0..self.dimensions.0
            {
                match self.positions[0].get(&(j as i32, i as i32))
                {
                    Some(x) => {
                        result = write!(f, "{:?}", self.cells[*x]);
                    },
                    None => {
                        result = write!(f, ".");
                    }
                }
            }
            result = write!(f, "\n");
        }
        result
    }
}


//
//
const TIER_ONE_CHAIN: u16 = 0xFF;
const TIER_TWO_CHAIN: u16  = 0x0F;
const TIER_THREE_CHAIN: u16 = 0x0F;
struct TieredGene
{
    tier: u8,
    trait_number: u16
}
impl TieredGene
{
    fn new(gene_payload: u16) -> TieredGene
    {
        //TODO: Make this configurable
        //TIER I
        let mut tier: u8 = 1;
        let mut trait_num = gene_payload & (0xFF<<8) as u16;
        if trait_num == TIER_ONE_CHAIN
        {
            tier += 1;
            //TIER II
            trait_num = (gene_payload & (0xF<<4)) as u16;
            if trait_num == TIER_TWO_CHAIN
            {
                //TIER III
                trait_num = (gene_payload & 0xF) as u16;
            }
        }

        TieredGene
        {
            tier: tier,
            trait_number: trait_num
        }
    }
}


//
//
trait PolyminiCellFactory
{
    fn create_for_gene(g: Gene) -> Cell;
}

struct BasicPolyminiCellFactory;
impl PolyminiCellFactory for BasicPolyminiCellFactory
{
    fn create_for_gene(g: Gene) -> Cell
    {
        let v = g.adjacency_payload;
        let dirs = vec![Directions::UP, Directions::DOWN, Directions::LEFT, Directions::RIGHT];
        let mut adj_dirs = vec![];

        for i in 0..4
        {
            if 1<<i & v != 0 
            {
                adj_dirs.push(dirs[i]);
            }
        }

        let ai = AdjacencyInfo { adj: adj_dirs };
         
        let tg = TieredGene::new(g.genetic_payload); 

        Cell::new(ai)
    }
}


//
//
#[derive(Debug)]
pub struct Morphology
{
    dimensions: (u8, u8),
    representations: Representation,
}
impl Morphology
{
    pub fn new() -> Morphology
    {
        Morphology { dimensions: (0,0),
                     representations: Representation::new(vec![], vec![], (0,0), (0,0)) }
    }

    //TODO: This names wtf XD
    fn construct(genes: Vec<Gene>, _:Option<i8>) -> Vec<Cell>
    {
        let mut to_ret = vec![];
        for g in genes
        {
            to_ret.push(BasicPolyminiCellFactory::create_for_gene(g));
        }
        to_ret
    }

    fn build(mut cells: Vec<Cell>) -> Morphology
    {
        let mut visited: HashSet<Coord> = HashSet::new();
        let mut stack: Vec<Coord> = Vec::new();
        let mut positions: Vec<Coord> = Vec::new();

        let mut minx =  1000;
        let mut maxx = -1000;
        let mut miny =  1000;
        let mut maxy = -1000;


        // Iterate through the cells gathering adjacency info
        let mut curr_coord = (0, 0);
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

                minx = min(minx, coord.0);
                miny = min(miny, coord.1);

                maxx = max(maxx, coord.0);
                maxy = max(maxy, coord.1);
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

        let w = (maxx - minx + 1) as u8;
        let h = (maxy - miny + 1) as u8;

        let r1 = Representation::new(drain_pos, drain_cell,
                                     (w, h), (minx, miny));
        
        Morphology { dimensions: (w, h), representations: r1 }
    }
}


//
//
#[cfg(test)]
mod test
{
    use ::morphology::*;
    #[test]
    fn test_adjacency_vertical()
    {
        let adj_info = AdjacencyInfo{ adj: vec![Directions::UP, Directions::DOWN] };
        let neighbours = adj_info.get_neighbours((0, 1));
        assert_eq!(neighbours.len(), 2);
        assert_eq!( (neighbours[0].0, neighbours[0].1), (0,0));
        assert_eq!( (neighbours[1].0, neighbours[1].1), (0,2));
    }

    #[test]
    fn test_adjacency_vertical_2()
    {
        let adj_info = AdjacencyInfo{ adj: vec![Directions::UP, Directions::DOWN] };
        let neighbours = adj_info.get_neighbours((0, 0));
        assert_eq!(neighbours.len(), 1);
        assert_eq!((neighbours[0].0, neighbours[0].1), (0,1));
    }

    #[test]
    fn test_dimensions_1()
    {
        let cells = vec![ Cell::new( AdjacencyInfo { adj: vec![Directions::LEFT, Directions::RIGHT] }),
                          Cell::new( AdjacencyInfo { adj: vec![] }),
                          Cell::new( AdjacencyInfo { adj: vec![] })];
        let morph = Morphology::build(cells);

        assert_eq!(morph.dimensions.0, 3);
        assert_eq!(morph.dimensions.1, 1);
    }

    #[test]
    fn test_representation()
    {
        let cells = vec![ Cell::new( AdjacencyInfo { adj: vec![Directions::LEFT, Directions::RIGHT] }),
                          Cell::new( AdjacencyInfo { adj: vec![] }),
                          Cell::new( AdjacencyInfo { adj: vec![] })];
        let morph = Morphology::build(cells);

        for i in 0..4
        {
            println!("--");
            for c in &morph.representations.positions[i]
            {
                println!("{:?}", c)
            }
        }
    }

    #[test]
    fn test_cell_parse()
    {
       let v1: u32 = 0xBE;
       let v2: u32 = 0xC5;
       let v3: u32 = 0x6AAD;
       let gene = Gene::new( (v1 << 24) + (v2 << 16) + v3 );

       assert_eq!(gene.control_payload, v1 as u8);
       assert_eq!(gene.adjacency_payload, v2 as u8);
       assert_eq!(gene.genetic_payload, v3 as u16);
    }

    #[test]
    fn test_gene_to_cell()
    {
        let v1: u32 = 0x09;
        let v2: u32 = 0x0B;
        let genes = vec![Gene::new((v1 << 16) + 0x6AAD),
                         Gene::new((v2 << 16) + 0xBEDA),
                         Gene::new(0xDEADBEEF), Gene::new(0x0600DBAD)];
        let cells = Morphology::construct(genes, None);

        let morph = Morphology::build(cells);
        for i in 0..4
        {
            println!("--");
            for c in &morph.representations.positions[i]
            {
                println!("{:?}", c)
            }
        }

        println!("{:?}", morph);
    }
}
