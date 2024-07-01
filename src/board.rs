use std::fmt::{Debug, Formatter, Result};
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
    move_balls: Vec<u8>,
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
            move_balls: Vec::new(),
        }
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

    pub fn add_actions(&mut self, s: &char) {
        self.actions.push(*s);
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
        self.move_balls.push(0);
    }

    pub fn remove_garbage_ball_by_idx(&mut self, idx: usize) {
        self.garbage_balls.remove(idx);
        self.move_balls.remove(idx);
    }

    pub fn move_ball(&mut self, from_idx: usize, to_idx: usize) {
        if let Some(i) = self.get_ball_id(from_idx) {
            self.garbage_balls[i] = to_idx;
            self.move_balls[i] += 1;
        }
    }

    pub fn get_ball_id(&self, idx: usize) -> Option<usize> {
        self.garbage_balls.iter().position(|&x| x == idx)
    }

    pub fn try_push(&self, board: &Board, garbage_ball_position: usize) -> Option<State> {
        let target_ball = garbage_ball_position * 2 - self.current_pos; // ball + (ball - me)

        // cannot move the ball more than N times
        if let Some(ball_id) = self.get_ball_id(garbage_ball_position) {
            if self.move_balls[ball_id] >= 2 {
                return None;
            }
        }

        // cannot push the ball to the target
        if target_ball == board.get_target() {
            return None;
        }

        // cannot push the ball to another garbage ball
        if self.is_garbage_ball(target_ball) {
            return None;
        }

        match board.get_cell(target_ball) {
            Cell::Wall => None,
            Cell::MagneticField(_) => {
                let mut new_state = self.clone();
                new_state.move_ball(garbage_ball_position, target_ball);
                Some(new_state)
            }
            Cell::Empty => {
                let mut new_state = self.clone();
                new_state.move_ball(garbage_ball_position, target_ball);
                Some(new_state)
            }
            Cell::Switch(id) => {
                let mut new_state = self.clone();
                new_state.move_ball(garbage_ball_position, target_ball);
                new_state.toggle_magnetic_field(*id);
                Some(new_state)
            }
        }
    }
}

impl Clone for State {
    fn clone(&self) -> State {
        State {
            current_pos: self.current_pos,
            garbage_balls: self.garbage_balls.clone(),
            actions: self.actions.clone(),
            magnetic_fields: self.magnetic_fields,
            move_balls: self.move_balls.clone(),
        }
    }
}

impl Debug for State {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "State {{ current_pos: {}, garbage_balls: {:?}, magnetic_fields: {}, actions: {} }}",
            self.current_pos, self.garbage_balls, self.magnetic_fields, self.actions
        )
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
    }
}

impl Eq for State {}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.current_pos == other.current_pos
            && self.magnetic_fields == other.magnetic_fields
            && self.garbage_balls == other.garbage_balls
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

    #[allow(dead_code)]
    pub fn show(&self, state: &State) {
        let x_start = self.start % 21;
        let y_start = self.start / 21;
        let x_target = self.target % 21;
        let y_target = self.target / 21;
        eprintln!(
            "Start: ({}, {}) -> Target: ({}, {})",
            x_start, y_start, x_target, y_target
        );
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

        for &idx in idx_to_change.iter().rev() {
            state.remove_garbage_ball_by_idx(idx);
        }
    }

    pub fn simplify(&mut self, state: &mut State) {
        self.simplify_balls(state);
        self.simplify_deadend();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec() {
        let v1 = vec![1, 2, 3];
        let v2 = vec![1, 3, 2];

        assert!(v1 == v2);
    }
}
