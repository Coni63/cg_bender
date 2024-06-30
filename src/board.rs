use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
    Wall,
    Switch(usize),
    MagneticField(usize),
    Empty,
}

pub struct State {
    current_pos: usize,
    garbage_balls: Vec<usize>,
    actions: String,
    magnetic_fields: u16,
}

impl State {
    pub fn new(position: usize) -> State {
        State {
            current_pos: position,
            garbage_balls: Vec::new(),
            actions: String::new(),
            magnetic_fields: 0,
        }
    }

    pub fn fitness(&self) -> usize {
        self.actions.len()
    }

    pub fn toggle_magnetic_field(&mut self, idx: usize) {
        self.magnetic_fields ^= 1 << idx;
    }

    pub fn is_magnetic_field_on(&self, idx: usize) -> bool {
        self.magnetic_fields & (1 << idx) != 0
    }

    pub fn get_actions(&self) -> &String {
        &self.actions
    }

    pub fn add_actions(&mut self, s: &String) {
        self.actions += s;
    }

    pub fn get_current_pos(&self) -> usize {
        self.current_pos
    }

    pub fn set_current_pos(&mut self, idx: usize) {
        self.current_pos = idx;
    }

    pub fn get_garbage_balls(&self) -> &Vec<usize> {
        &self.garbage_balls
    }

    pub fn is_garbage_ball(&self, idx: usize) -> bool {
        self.garbage_balls.contains(&idx)
    }

    pub fn add_garbage_ball(&mut self, idx: usize) {
        self.garbage_balls.push(idx);
    }

    pub fn remove_garbage_ball(&mut self, idx: usize) {
        if let Some(pos) = self.garbage_balls.iter().position(|&x| x == idx) {
            self.garbage_balls.remove(pos);
        }
    }

    pub fn clone(&self) -> State {
        State {
            current_pos: self.current_pos,
            garbage_balls: self.garbage_balls.clone(),
            actions: self.actions.clone(),
            magnetic_fields: self.magnetic_fields,
        }
    }
}

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash each field in the struct
        self.current_pos.hash(state);
        for ball in &self.garbage_balls {
            ball.hash(state);
        }
        self.magnetic_fields.hash(state);

        // eprintln!(
        //     "{} {:?} {}",
        //     self.current_pos, self.garbage_balls, self.magnetic_fields
        // );
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        (other.fitness()).cmp(&self.fitness())
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for State {}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.fitness() == other.fitness()
            && self.magnetic_fields == other.magnetic_fields
            && self.current_pos == other.current_pos
    }
}

pub struct Board {
    board: [Cell; 441],
    start: usize,
    target: usize,
    all_switches: [usize; 11],
    all_magnetic_fields: [usize; 11],
}

impl Board {
    pub fn new() -> Board {
        Board {
            board: [Cell::Wall; 441],
            start: 0,
            target: 0,
            all_switches: [0; 11],        // index of the switch
            all_magnetic_fields: [0; 11], // index of the magnetic field
        }
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: Cell) {
        let pos = y * 21 + x;
        match cell {
            Cell::Switch(idx) => self.all_switches[idx] = pos,
            Cell::MagneticField(idx) => self.all_magnetic_fields[idx] = pos,
            _ => (),
        }
        self.board[pos] = cell;
    }

    pub fn get_cell(&self, idx: usize) -> &Cell {
        &self.board[idx]
    }

    pub fn get_all_switches(&self) -> [usize; 11] {
        self.all_switches
    }

    pub fn get_all_magnetic_fields(&self) -> [usize; 11] {
        self.all_magnetic_fields
    }

    pub fn set_start(&mut self, x: usize, y: usize) {
        self.start = y * 21 + x;
    }

    pub fn get_start(&self) -> usize {
        self.start
    }

    pub fn set_target(&mut self, x: usize, y: usize) {
        self.target = y * 21 + x;
    }

    pub fn get_target(&self) -> usize {
        self.target
    }

    pub fn show(&self, state: &State) {
        for y in 0..21 {
            for x in 0..21 {
                let idx = y * 21 + x;
                if state.is_garbage_ball(idx) {
                    eprint!("+");
                } else {
                    match self.get_cell(idx) {
                        Cell::Wall => eprint!("#"),
                        Cell::Switch(_) => eprint!("S"),
                        Cell::MagneticField(_) => eprint!("M"),
                        Cell::Empty => eprint!("."),
                    }
                }
            }
            eprintln!();
        }
    }

    fn simplify_deadend(&mut self) {
        let offset = [-1, 1, -21, 21];

        let mut improved = true;
        while improved {
            improved = false;
            for y in 1..20 {
                for x in 1..20 {
                    let idx = y * 21 + x;
                    if idx == self.start || idx == self.target {
                        continue;
                    }

                    if self.board[idx] == Cell::Empty {
                        let mut count = 0;
                        for &o in offset.iter() {
                            let new_idx = (idx as i32 + o) as usize;
                            if self.board[new_idx] == Cell::Wall {
                                count += 1;
                            }
                        }
                        if count == 3 {
                            self.board[idx] = Cell::Wall;
                            improved = true;
                        }
                    }
                }
            }
        }
    }

    fn simplify_balls(&mut self, state: &mut State) {
        let mut idx_to_change = vec![];
        let corners = [(-1, -21), (1, -21), (-1, 21), (1, 21)];
        for (i, &ball) in state.get_garbage_balls().iter().enumerate() {
            for (corner1, corner2) in corners.iter() {
                let idx1 = (ball as i32 + corner1) as usize;
                let idx2 = (ball as i32 + corner2) as usize;
                if (self.board[idx1] == Cell::Wall) && (self.board[idx2] == Cell::Wall) {
                    self.board[ball] = Cell::Wall;
                    idx_to_change.push(i);
                    break;
                }
            }
        }

        for &idx in idx_to_change.iter() {
            state.remove_garbage_ball(idx);
        }
    }

    pub fn simplify(&mut self, state: &mut State) {
        self.simplify_balls(state);
        self.simplify_deadend();
    }
}
