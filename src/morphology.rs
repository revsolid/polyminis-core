use std::collections::{HashSet, HashMap};
use std::cmp::{min, max};

//
//
type Coord = (i32, i32);

pub enum Directions
{
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

//
//
struct Gene // ?
{
}
impl Gene
{ 
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
                Directions::UP    => to_ret.push( (coord.0,     coord.1 + 1)),
                Directions::DOWN  => to_ret.push( (coord.0,     coord.1 - 1)),
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
    coord: Coord,
}
impl Cell
{
    fn new(adjacency_info: AdjacencyInfo) -> Cell
    {
        Cell::new_with_coord(adjacency_info, (0,0))
    }

    fn new_with_coord(adjacency_info: AdjacencyInfo, coord: Coord) -> Cell
    {
        Cell { adjacency_info: adjacency_info, coord: coord }
    }

    fn set_coord(&mut self, c: Coord)
    {
        self.coord = c;
    }
}


//
//
struct Representation
{
    cells: Vec<Cell>,
}
impl Representation
{
    pub fn new(positions: Vec<Coord>, cells: Vec<Cell>) -> Representation
    {
        for position in positions
        {
        }
        Representation { cells: cells }
    }

    pub fn rotate(&self) -> Representation
    {
        Representation { cells: vec![] }
    }
}


//
//
pub struct Morphology
{
    dimensions: Coord,
    representations: [Representation; 4],
}
impl Morphology
{
    pub fn new() -> Morphology
    {
        Morphology { dimensions: (0,0) ,
                     representations: [ Representation { cells: vec![] },
                                        Representation { cells: vec![] },
                                        Representation { cells: vec![] },
                                        Representation { cells: vec![] }] }
    }

    fn parse(genes: Vec<Gene>, factory: &PolyminiCellFactory) -> Vec<Cell>
    {
        vec![]
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
            cell.set_coord(curr_coord);
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

                if coord.0 < minx
                {
                    minx = coord.0;
                }
                if coord.0 > maxx
                {
                    maxx = coord.0;
                }
                if coord.1 < miny
                {
                    miny = coord.1;
                }
                if coord.1 > maxy
                {
                    maxy = coord.1;
                }
            }
            match stack.pop()
            {
                Some(c) => { curr_coord = c; },
                None => { break; }
            }
        }

        let l = min(cells.len(), positions.len());
        let mut drain: Vec<Cell> = cells.drain(0..l).collect();
        drain.reverse();

        let w  = maxx - minx;
        let h  = maxy - miny;

        // Re-Iterate through the cells and placing them in the [0] representation
        let r1 = Representation::new(positions, drain);
        
        // Create the other representation for the other 3 possible
        // orientations
        // TODO: This needs to come from the previous code, not trivial 

        Morphology { dimensions: (w, h), representations: [r1,
                                                           Representation { cells: vec![] },
                                                           Representation { cells: vec![] },
                                                           Representation { cells: vec![] }] }
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
    fn test_adjacency()
    {
        let adj_info = AdjacencyInfo{ adj: vec![Directions::UP, Directions::DOWN] };
        let neighbours = adj_info.get_neighbours((0, 0));
        assert_eq!(neighbours.len(), 2);
        assert_eq!(neighbours[0].0, 0);
        assert_eq!(neighbours[0].1, 1);
        assert_eq!(neighbours[1].0, 0);
        assert_eq!(neighbours[1].1, -1);
    }

    #[test]
    fn test_dimensions_1()
    {
        let cells = vec![ Cell::new( AdjacencyInfo { adj: vec![Directions::LEFT, Directions::RIGHT] }),
                          Cell::new( AdjacencyInfo { adj: vec![] }),
                          Cell::new( AdjacencyInfo { adj: vec![] })];
        let morph = Morphology::build(cells);

        assert_eq!(morph.dimensions.0, 2);
        assert_eq!(morph.dimensions.1, 0);
        assert_eq!(morph.representations[0].cells[2].coord.0,  0);
        assert_eq!(morph.representations[0].cells[2].coord.1,  0);
        assert_eq!(morph.representations[0].cells[1].coord.0,  1);
        assert_eq!(morph.representations[0].cells[1].coord.1,  0);
        assert_eq!(morph.representations[0].cells[0].coord.0, -1);
        assert_eq!(morph.representations[0].cells[0].coord.1,  0);
    }
}
