use std::collections::{HashSet};
use std::cmp::{min, max};

//
//
pub type Coord = (i32, i32);

#[derive(Copy, Clone)]
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

        println!("CP: {} AP: {} GP: {}", cp, ap, gp);

        Gene { control_payload: cp,
               adjacency_payload: ap,
               genetic_payload: gp,
             }
    }
}


//
//
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


//
//
struct Representation
{
    cells: Vec<Cell>,
    positions: [Vec<Coord>; 4]
}
impl Representation
{
    pub fn new(positions: Vec<Coord>, cells: Vec<Cell>) -> Representation
    {
        let mut all_positions : [Vec<Coord>; 4] = [vec![], vec![], vec![], vec![]];

        all_positions[0] = positions;
        all_positions[1] = Representation::rotate(&all_positions[0]); 
        all_positions[2] = Representation::rotate(&all_positions[1]); 
        all_positions[3] = Representation::rotate(&all_positions[2]); 

        Representation { cells: cells, positions: all_positions }
    }

    fn rotate(positions: &Vec<Coord>) -> Vec<Coord>
    {
        let mut to_ret = vec![];
        for c in positions 
        {
            // Rotation matrix [ cosA  -sinA ]
            //                 [ sinA   cosA  ]
            // A = 90 degrees
            to_ret.push( (c.1, -1 * c.0) );
        }
        to_ret
    }
}


//
//
pub struct Morphology
{
    dimensions: Coord,
    representations: Representation,
}
impl Morphology
{
    pub fn new() -> Morphology
    {
        Morphology { dimensions: (0,0),
                     representations: Representation { cells: vec![], positions: [vec![], vec![], vec![], vec![]] } }
    }

    //TODO: This names wtf XD
    fn construct(genes: Vec<Gene>, _: Option<i32>) -> Vec<Cell>
    {
        let mut to_ret = vec![];
        for g in genes
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

            to_ret.push(Cell::new(ai));
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

        let w  = maxx - minx + 1;
        let h  = maxy - miny + 1;

        // Re-Iterate through the cells and placing them in the [0] representation
        let r1 = Representation::new(drain_pos, drain_cell);
        
        Morphology { dimensions: (w, h), representations: r1 }
    }
}


//
//
trait PolyminiCellFactory
{
    fn create_for_gene(&self, g: Gene) -> Cell;
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
        let genes = vec![Gene::new(v1 << 16),
                         Gene::new(v2 << 16),
                         Gene::new(0), Gene::new(0)];
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
    }
}
