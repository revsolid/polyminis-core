use std::collections::{HashSet, HashMap};
use std::cmp::{min, max};
use std::fmt;

use rust_monster::ga::ga_random::*;

use ::genetics::Genetics;

//
//
pub type Coord = (i32, i32);
pub type Chromosome = [u8; 4];

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
    tiered_gene: TieredGene
}
impl Cell
{
    fn new(adjacency_info: AdjacencyInfo, tiered_gene: TieredGene) -> Cell
    {
        Cell { adjacency_info: adjacency_info, tiered_gene: tiered_gene }
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
// Positions - Maps a Coordinate to a position in the Cell list
const TOTAL_ORIENTATIONS: usize = 4; 
struct Representation
{
    cells: Vec<Cell>,
    positions: [HashMap<Coord, usize>; TOTAL_ORIENTATIONS],
    dimensions: (u8, u8),
    corners: [(i32, i32); TOTAL_ORIENTATIONS],
}
impl Representation
{
    pub fn new(positions: Vec<Coord>, cells: Vec<Cell>,
               dimensions: (u8, u8), corner: (i32, i32)) -> Representation
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
        let maxx = dimensions.0 as i32 + minx - 1;
        let maxy = dimensions.1 as i32 + miny - 1;

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

            result = write!(f, "\n");
            for i in 0..h 
            {
                for j in 0..w 
                {
                    let mut trans_coord = (j as i32, i as i32);
                    trans_coord.0 += self.corners[p].0;
                    trans_coord.1 += self.corners[p].1;

                    match self.positions[p].get(&trans_coord)
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
        }
        result
    }
}


//
//
const TIER_ONE_CHAIN: u16 = 0xFF;
const TIER_TWO_CHAIN: u16  = 0x0F;
const TIER_THREE_CHAIN: u16 = 0x0F;
pub struct TieredGene
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
    fn create_for_chromosome(c: Chromosome) -> Cell;
}

struct BasicPolyminiCellFactory;
impl PolyminiCellFactory for BasicPolyminiCellFactory
{
    fn create_for_chromosome(chromosome: Chromosome) -> Cell
    {

        //TODO: Control / Metadata Payload
        let _ = 0xFF & chromosome[0];
        let ap = 0xFF & chromosome[1];
        let gp1 = ((0xFF & chromosome[2]) as u16) << 8;
        let gp  = gp1 + (0xFF & chromosome[3]) as u16;

        let dirs = vec![Directions::UP, Directions::DOWN, Directions::LEFT, Directions::RIGHT];
        let mut adj_dirs = vec![];

        for i in 0..4
        {
            if 1<<i & ap != 0 
            {
                adj_dirs.push(dirs[i]);
            }
        }

        let ai = AdjacencyInfo { adj: adj_dirs };
         
        let tg = TieredGene::new(gp); 

        Cell::new(ai, tg)
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

    fn new(chromosomes: Vec<Chromosome>) -> Morphology
    {
        let mut cells = vec![];
        let mut original_chromosome = vec![];
        for c in chromosomes
        {
            cells.push(BasicPolyminiCellFactory::create_for_chromosome(c));
            original_chromosome.push(c);
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

        let r1 = Representation::new(drain_pos, drain_cell,
                                     (w, h), (minx, miny));
        
        Morphology { dimensions: (w, h), representations: r1, original_chromosome: original_chromosome}
    }
}
impl Genetics for Morphology
{
    fn crossover(&self, other: &Morphology, random_ctx: &mut GARandomCtx) -> Morphology
    {
        let mut chromosomes = vec![];

        // TODO: A LOT of magic numbers :S
        // bit to make the cut at
        let cross_point_chromosome = random_ctx.gen_range(0, self.original_chromosome.len());
        let cross_point_allele = random_ctx.gen_range(0, 4);
        let cross_point_bit = random_ctx.gen_range(0, 8);


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

        let mut link_byte = 0;

        // make the mask u16 to allow space for overflowing
        let mut mask : u16 = ( 1 << (cross_point_bit+1)) - 1;
        mask = (mask << (8 - cross_point_bit));

        let mask_2 : u16 = (1 << ((8 - cross_point_bit))) - 1;

        let cross_point_chromosome_2 = random_ctx.gen_range(0, other.original_chromosome.len()); 

        for lc in cross_point_allele..4
        {
            link_chromosome[lc] = other.original_chromosome[cross_point_chromosome][lc];
        }

        println!("{:X} {:X}", mask as u8, mask_2 as u8);
        link_byte = (mask as u8)   & (self.original_chromosome[cross_point_chromosome][cross_point_allele]) +
                    (mask_2 as u8) & (other.original_chromosome[cross_point_chromosome_2]
                                                               [cross_point_allele]);
        link_chromosome[cross_point_allele] = link_byte;
        chromosomes.push(link_chromosome);


        for j in cross_point_chromosome_2 + 1..other.original_chromosome.len()
        {
            chromosomes.push([ other.original_chromosome[j][0],
                               other.original_chromosome[j][1],
                               other.original_chromosome[j][2],
                               other.original_chromosome[j][3] ])
        }

        println!("{} {} {} {}", cross_point_chromosome, cross_point_allele, cross_point_bit, cross_point_chromosome_2);
        println!("{:?} {:?} {:?}", link_chromosome, self.original_chromosome[cross_point_chromosome], other.original_chromosome[cross_point_chromosome_2]);
        println!("{}", link_byte);

        Morphology::new(chromosomes)
    }

    fn mutate(&self, _: &mut GARandomCtx){}
}

//
//
#[cfg(test)]
mod test
{
    use rust_monster::ga::ga_random::*;
    use ::morphology::*;
    use ::genetics::*;
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

        let morph = Morphology::new(chromosomes);

        assert_eq!((3, 2), (morph.dimensions.0, morph.dimensions.1));
        assert_eq!(morph.original_chromosome[0][1], v1);
        assert_eq!(morph.original_chromosome[1][1], v2);
        println!("{:?}", morph);
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

        let morph = Morphology::new(chromosomes);
        assert_eq!((3, 1), (morph.dimensions.0, morph.dimensions.1));
        assert_eq!(morph.original_chromosome[0][1], v1);
        println!("{:?}", morph);
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

        let morph = Morphology::new(c1);
        let morph_2 = Morphology::new(c2);

        let child = morph.crossover(&morph_2, &mut GARandomCtx::from_seed([5,7,5,9], "".to_string())); 

        println!("{:?}", child);
    }
}
