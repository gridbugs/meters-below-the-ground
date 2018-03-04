use std::mem;
use std::collections::{VecDeque, HashSet};
use rand::Rng;
use direction::*;
use entity_store::EntityIdAllocator;
use super::*;
use prototypes;

pub fn size() -> Size {
    Size::new(29, 29)
}

const ROOM_PLACEMENT_ATTEMPTS: usize = 100;
const PRELIM_ROOM_MIN_SIZE: u32 = 3;
const PRELIM_ROOM_MAX_SIZE: u32 = 4;

fn random_between_inclusive<R: Rng>(min: u32, max: u32, rng: &mut R) -> u32 {
    let delta = max - min;
    let random_delta = rng.gen::<u32>() % (delta + 1);
    min + random_delta
}

#[derive(Debug)]
struct PrelimRoom {
    position: Coord,
    size: Size,
}

impl PrelimRoom {
    fn room<R: Rng>(&self, rng: &mut R) -> Room {

        use self::CardinalDirection::*;
        let mut door_sides = vec![North, East, South, West];
        rng.shuffle(&mut door_sides);
        let sides_to_remove = random_between_inclusive(1, 2, rng);
        for _ in 0..sides_to_remove {
            door_sides.pop();
        }

        let doors = door_sides.iter().map(|&d| {
            let coord = match d {
                North => {
                    let offset = random_between_inclusive(1, self.size.x() - 2, rng) as i32;
                    Coord::new(offset, 0)
                }
                South => {
                    let offset = random_between_inclusive(1, self.size.x() - 2, rng) as i32;
                    Coord::new(offset, self.size.y() as i32 - 1)
                }
                East => {
                    let offset = random_between_inclusive(1, self.size.y() - 2, rng) as i32;
                    Coord::new(self.size.x() as i32 - 1, offset)
                }
                West => {
                    let offset = random_between_inclusive(1, self.size.y() - 2, rng) as i32;
                    Coord::new(0, offset)
                }
            };
            // covert coordinate system
            let coord = coord + coord;
            (d, coord)
        }).collect();

        Room {
            size: self.size + self.size - Size::new(1, 1),
            position: self.position + self.position,
            doors,
        }
    }
}

#[derive(Debug)]
struct Room {
    position: Coord,
    size: Size,
    doors: Vec<(CardinalDirection, Coord)>,
}

impl Room {
    fn centre(&self) -> Coord {
        self.position + Size::new(self.size.x() / 2, self.size.y() / 2)
    }
}

fn choose_rooms<R: Rng>(rng: &mut R) -> Vec<Room> {
    let prelim_grid_size = Size::new((size().x() + 1) / 2, (size().y() + 1) / 2);
    let mut prelim_grid: Grid<bool> = Grid::new_default(prelim_grid_size);
    let mut rooms = Vec::new();

    for _ in 0..ROOM_PLACEMENT_ATTEMPTS {
        let width = random_between_inclusive(PRELIM_ROOM_MIN_SIZE, PRELIM_ROOM_MAX_SIZE, rng);
        let height = random_between_inclusive(PRELIM_ROOM_MIN_SIZE, PRELIM_ROOM_MAX_SIZE, rng);
        let size = Size::new(width, height);

        let max_x = prelim_grid_size.x() - size.x();
        let max_y = prelim_grid_size.y() - size.y();
        let x = random_between_inclusive(1, max_x - 1, rng);
        let y = random_between_inclusive(1, max_y - 1, rng);
        let position = Coord::new(x as i32, y as i32);
        let room = PrelimRoom {
            position,
            size,
        };

        let mut valid = true;
        for coord in size.coords().map(|c| c + position) {
            let cell = prelim_grid.get_mut(coord).unwrap();
            if *cell {
                valid = false;
                break;
            }
            *cell = true;
        }

        if valid {
            rooms.push(room.room(rng));
        }
    }

    rooms
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Door {
    Present,
    Absent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    RoomWall,
    CavernWall,
    Floor,
    Doorway(CardinalDirection, Door),
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Floor
    }
}

fn place_rooms<R: Rng>(grid: &mut Grid<Cell>, rooms: &Vec<Room>, rng: &mut R) -> Vec<(Coord, CardinalDirection)> {
    let mut doors = Vec::new();

    for room in rooms.iter() {

        for coord in room.size.coords().map(|c| c + room.position) {
            let cell = grid.get_mut(coord).unwrap();
            *cell = Cell::Floor;
        }

        for i in 0..room.size.x() {
            let coord = room.position + Coord::new(i as i32, 0);
            {
                let cell = grid.get_mut(coord).unwrap();
                *cell = Cell::RoomWall;
            }
            let coord = room.position + Coord::new(i as i32, room.size.y() as i32 - 1);
            {
                let cell = grid.get_mut(coord).unwrap();
                *cell = Cell::RoomWall;
            }
        }
        for i in 1..(room.size.y() - 1) {
            let coord = room.position + Coord::new(0, i as i32);
            {
                let cell = grid.get_mut(coord).unwrap();
                *cell = Cell::RoomWall;
            }
            let coord = room.position + Coord::new(room.size.x() as i32 - 1, i as i32);
            {
                let cell = grid.get_mut(coord).unwrap();
                *cell = Cell::RoomWall;
            }
        }

        for &(direction, coord) in room.doors.iter() {
            let door_present: bool = rng.gen();
            let door = if door_present {
                Door::Present
            } else {
                Door::Absent
            };
            let coord = room.position + coord;
            let cell = grid.get_mut(coord).unwrap();
            *cell = Cell::Doorway(direction, door);

            doors.push((coord, direction));
        }
    }

    doors
}

#[derive(Debug, Clone, Copy, Default)]
struct ConwayCell {
    alive: bool,
    next_alive: bool,
    post_processed: bool,
}

const NUM_CAVERN_STEPS: usize = 6;
const CAVERN_SURVIVE_MIN: usize = 4;
const CAVERN_SURVIVE_MAX: usize = 8;
const CAVERN_RESURRECT_MIN: usize = 5;
const CAVERN_RESURRECT_MAX: usize = 5;
const CAVERN_SIZE_THRESHOLD: usize = 12;

fn place_caverns<R: Rng>(grid: &mut Grid<Cell>, rng: &mut R) {
    let mut conway_grid: Grid<ConwayCell> = Grid::new_default(size());
    for cell in conway_grid.iter_mut() {
        cell.alive = rng.gen();
    }

    let width = conway_grid.width();
    let height = conway_grid.height();

    for _ in 0..NUM_CAVERN_STEPS {
        for coord in conway_grid.coords() {
            if coord.x == 0 || coord.y == 0 || coord.x == width as i32 - 1 || coord.y == height as i32 - 1 {
                let cell = conway_grid.get_mut(coord).unwrap();
                cell.next_alive = true;
            } else {
                let mut count = 0;
                for d in Directions {
                    let coord = d.coord() + coord;
                    let neighbour = conway_grid.get(coord).unwrap();
                    if neighbour.alive {
                        count += 1;
                    }
                }

                let cell = conway_grid.get_mut(coord).unwrap();
                if cell.alive {
                    cell.next_alive = count >= CAVERN_SURVIVE_MIN && count <= CAVERN_SURVIVE_MAX;
                } else {
                    cell.next_alive = count >= CAVERN_RESURRECT_MIN && count <= CAVERN_RESURRECT_MAX;
                }
            }
        }

        for cell in conway_grid.iter_mut() {
            cell.alive = cell.next_alive;
        }
    }

    for coord in conway_grid.coords() {
        let should_process = {
            let cell = conway_grid.get(coord).unwrap();
            !cell.alive && !cell.post_processed
        };
        if should_process {
            let mut seen = Vec::new();
            let mut to_visit = vec![coord];

            while let Some(coord) = to_visit.pop() {
                seen.push(coord);
                conway_grid.get_mut(coord).unwrap().post_processed = true;
                for d in CardinalDirections {
                    let next = coord + d.coord();

                    let should_process = {
                        let cell = conway_grid.get(next).unwrap();
                        !cell.alive && !cell.post_processed
                    };
                    if should_process {
                        to_visit.push(next);
                    }
                }
            }

            if seen.len() < CAVERN_SIZE_THRESHOLD {
                for coord in seen {
                    conway_grid.get_mut(coord).unwrap().alive = true;
                }
            }
        }
    }

    for (conway_cell, world_cell) in izip!(conway_grid.iter(), grid.iter_mut())  {
        if conway_cell.alive {
            *world_cell = Cell::CavernWall;
        } else {
            *world_cell = Cell::Floor;
        }
    }

}

#[derive(Debug, Clone, Copy)]
enum Visited {
    Initial,
    Step(CardinalDirection),
}

fn door_dig(grid: &mut Grid<Cell>, doors: &Vec<(Coord, CardinalDirection)>) {

    for &(coord, direction) in doors.iter() {
        let outside_door_coord = coord + direction.coord();
        if *grid.get(outside_door_coord).unwrap() != Cell::Floor {
            let mut visited: Grid<Option<Visited>> = Grid::new_clone(size(), None);
            let mut open_set = VecDeque::new();
            open_set.push_back(outside_door_coord);
            *visited.get_mut(outside_door_coord).unwrap() = Some(Visited::Initial);
            let mut dest = None;
            'outer: while let Some(coord) = open_set.pop_front() {
                for d in CardinalDirections {
                    let neighbour_coord = coord + d.coord();
                    if let Some(cell) = grid.get(neighbour_coord).cloned() {
                        if visited.get(neighbour_coord).unwrap().is_some() {
                            continue;
                        }
                        if cell == Cell::Floor {
                            *visited.get_mut(neighbour_coord).unwrap() = Some(Visited::Step(d.opposite()));
                            dest = Some(neighbour_coord);
                            break 'outer;
                        }
                        if cell == Cell::CavernWall {
                            *visited.get_mut(neighbour_coord).unwrap() = Some(Visited::Step(d.opposite()));
                            open_set.push_back(neighbour_coord);
                        }
                    }
                }
            }
            if let Some(dest) = dest {
                let mut coord = dest;
                loop {
                    *grid.get_mut(coord).unwrap() = Cell::Floor;
                    let &visited = visited.get(coord).unwrap().as_ref().unwrap();
                    match visited {
                        Visited::Initial => break,
                        Visited::Step(direction) => coord += direction.coord(),
                    }
                }
            }
        }
    }
}

const PRUNE_SIZE_THRESHOLD: usize = 12;

fn prune_small_areas(grid: &mut Grid<Cell>) {

    let mut processed: Grid<bool> = Grid::new_default(size());

    for coord in grid.coords() {
        if *grid.get(coord).unwrap() == Cell::Floor &&
            !processed.get(coord).unwrap() {

            let mut seen = Vec::new();
            let mut to_visit = vec![coord];

            while let Some(coord) = to_visit.pop() {
                seen.push(coord);
                *processed.get_mut(coord).unwrap() = true;

                for d in CardinalDirections {
                    let next = coord + d.coord();
                    if *grid.get(next).unwrap() == Cell::Floor &&
                        !processed.get(next).unwrap() {

                        to_visit.push(next);
                    }
                }
            }

            if seen.len() < PRUNE_SIZE_THRESHOLD {
                for coord in seen {
                    *grid.get_mut(coord).unwrap() = Cell::CavernWall;
                }
            }
        }
    }
}

fn identify_largest_contiguous_space(grid: &Grid<Cell>) -> Vec<Coord> {
    let mut largest: Vec<Coord> = Vec::new();
    let mut processed: Grid<bool> = Grid::new_default(size());

    let is_candidate = |cell| {
        match cell {
            Cell::Floor | Cell::Doorway(_, _) => true,
            _ => false,
        }
    };

    for coord in grid.coords() {
        if !processed.get(coord).unwrap() &&
            is_candidate(*grid.get(coord).unwrap()) {

            let mut seen = Vec::new();
            let mut to_visit = vec![coord];
            *processed.get_mut(coord).unwrap() = true;

            while let Some(coord) = to_visit.pop() {
                seen.push(coord);

                for d in CardinalDirections {
                    let next = coord + d.coord();
                    if !processed.get(next).unwrap() &&
                        is_candidate(*grid.get(next).unwrap()) {

                        to_visit.push(next);
                        *processed.get_mut(next).unwrap() = true;
                    }
                }
            }

            if seen.len() > largest.len() {
                mem::swap(&mut seen, &mut largest);
            }
        }
    }

    largest
}

pub fn populate<R: Rng>(
    config: TerrainConfig,
    id_allocator: &mut EntityIdAllocator,
    messages: &mut MessageQueues,
    rng: &mut R,
) -> bool {

    let mut grid: Grid<Cell> = Grid::new_default(size());

    place_caverns(&mut grid, rng);

    let rooms = choose_rooms(rng);
    let mut doors = place_rooms(&mut grid, &rooms, rng);
    rng.shuffle(&mut doors);

    prune_small_areas(&mut grid);
    door_dig(&mut grid, &doors);

    let mut largest_space = identify_largest_contiguous_space(&grid);
    rng.shuffle(&mut largest_space);

    let room_centres = rooms.iter().map(|r| r.centre()).collect::<HashSet<_>>();
    let room_centres_in_largest_space = largest_space.iter().cloned().filter(|coord| room_centres.contains(coord)).collect::<Vec<_>>();

    if room_centres_in_largest_space.len() < 2 {
        return false;
    }

    let player_coord = room_centres_in_largest_space[0];
    let stairs_coord = room_centres_in_largest_space[1];

    for (coord, &cell) in grid.enumerate() {
        match cell {
            Cell::RoomWall => {
                prototypes::wall(id_allocator.allocate(), coord, messages);
            }
            Cell::CavernWall => {
                prototypes::cavern_wall(id_allocator.allocate(), coord, messages);
            }
            Cell::Floor => {
                prototypes::floor(id_allocator.allocate(), coord, messages);
            }
            Cell::Doorway(_, door) => {
                match door {
                    Door::Present => {
                        prototypes::door(id_allocator.allocate(), coord, messages);
                    }
                    Door::Absent => {
                        prototypes::floor(id_allocator.allocate(), coord, messages);
                    }
                }
            }
        }
    }

    prototypes::player(id_allocator.allocate(), player_coord, messages);

    if config.final_level {
        prototypes::exit(id_allocator.allocate(), stairs_coord, messages);
    } else {
        prototypes::stairs(id_allocator.allocate(), stairs_coord, messages);
    }

    true
}
