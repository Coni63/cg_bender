use std::collections::{HashSet, VecDeque};

use crate::board::{Board, Cell, State};

pub fn solve(board: &Board, initial_state: &State) -> Option<State> {
    let offset: [(i32, char); 4] = [(-1, 'L'), (1, 'R'), (-21, 'U'), (21, 'D')];

    let mut queue: VecDeque<State> = VecDeque::new();
    let mut visited: HashSet<State> = HashSet::new();

    queue.push_back(initial_state.clone());

    let start_time = std::time::Instant::now();
    while start_time.elapsed().as_millis() < 2500 {
        match queue.pop_front() {
            None => return None,
            Some(current_state) => {
                if current_state.get_current_pos() == board.get_target() {
                    return Some(current_state.clone());
                }

                if visited.contains(&current_state) {
                    continue;
                }
                visited.insert(current_state.clone());

                // explore the neighbors
                for (offset, direction) in offset.iter() {
                    let new_idx = (current_state.get_current_pos() as i32 + offset) as usize;

                    match board.get_cell(new_idx) {
                        Cell::Switch(id) => {
                            if current_state.is_garbage_ball(new_idx) {
                                match current_state.try_push(board, new_idx) {
                                    None => continue,
                                    Some(mut new_state) => {
                                        new_state.toggle_magnetic_field(*id);
                                        new_state.add_actions(direction);
                                        new_state.set_current_pos(new_idx);
                                        queue.push_back(new_state);
                                    }
                                }
                            } else {
                                let mut new_state = current_state.clone();
                                new_state.toggle_magnetic_field(*id);
                                new_state.add_actions(direction);
                                new_state.set_current_pos(new_idx);
                                queue.push_back(new_state);
                            }
                        }
                        Cell::MagneticField(id) => {
                            if current_state.is_magnetic_field_on(*id) {
                                continue;
                            }
                            let mut new_state = current_state.clone();
                            new_state.add_actions(direction);
                            new_state.set_current_pos(new_idx);
                            queue.push_back(new_state);
                        }
                        Cell::Empty => {
                            if current_state.is_garbage_ball(new_idx) {
                                match current_state.try_push(board, new_idx) {
                                    None => continue,
                                    Some(mut new_state) => {
                                        new_state.add_actions(direction);
                                        new_state.set_current_pos(new_idx);
                                        queue.push_back(new_state);
                                    }
                                }
                            } else {
                                let mut new_state = current_state.clone();
                                new_state.add_actions(direction);
                                new_state.set_current_pos(new_idx);
                                queue.push_back(new_state);
                            }
                        }
                        Cell::Wall => continue,
                    }
                }
            }
        }
    }
    None
}
