#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
    Wall,
    Switch(usize),
    MagneticField(usize),
    Empty,
}

pub struct State {
    garbage_balls: [usize; 10],
    actions: String,
    magnetic_fields: [bool; 11],
}

impl State {
    pub fn new() -> State {
        State {
            garbage_balls: [0; 10], // 0 is always a wall so we can use it as a sentinel
            actions: String::new(),
            magnetic_fields: [false; 11],
        }
    }

    pub fn fitness(&self) -> i32 {
        self.actions.len() as i32
    }

    pub fn hash(&self) -> u64 {
        let mut result = 0;
        for (i, &val) in self.magnetic_fields.iter().rev().enumerate() {
            if val {
                result |= 1 << i;
            }
        }
        result
    }

    pub fn add_garbage_ball(&mut self, idx: usize) {
        for i in 0..10 {
            if self.garbage_balls[i] == 0 {
                self.garbage_balls[i] = idx;
                return;
            }
        }
    }

    pub fn set_magnetic_field(&mut self, idx: usize, val: bool) {
        self.magnetic_fields[idx] = val;
    }

    pub fn get_actions(&self) -> String {
        self.actions.clone()
    }
}

pub struct Board {
    board: [Cell; 441],
    start: usize,
    target: usize,
}

impl Board {
    pub fn new() -> Board {
        Board {
            board: [Cell::Wall; 441],
            start: 0,
            target: 0,
        }
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: Cell) {
        self.board[y * 21 + x] = cell;
    }

    pub fn get_cell(&self, x: usize, y: usize) -> &Cell {
        &self.board[y * 21 + x]
    }

    pub fn get_cell_idx(&mut self, idx: usize) -> &Cell {
        &mut self.board[idx]
    }

    pub fn set_start(&mut self, x: usize, y: usize) {
        self.start = y * 21 + x;
    }

    pub fn set_target(&mut self, x: usize, y: usize) {
        self.target = y * 21 + x;
    }

    pub fn show(&self) {
        for y in 0..21 {
            for x in 0..21 {
                match self.get_cell(x, y) {
                    Cell::Wall => print!("#"),
                    Cell::Switch(_) => print!("S"),
                    Cell::MagneticField(_) => print!("M"),
                    Cell::Empty => print!("."),
                }
            }
            eprintln!();
        }
    }

    pub fn simplify(&mut self) {
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
}
