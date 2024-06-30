use std::collections::{HashSet, VecDeque};

use crate::board::{Board, Cell, State};

pub fn solve(board: &Board, initial_state: &State) -> Option<State> {
    let offset: [(i32, char); 4] = [(-1, 'L'), (1, 'R'), (-21, 'U'), (21, 'D')];

    let mut queue: VecDeque<State> = VecDeque::new();
    let mut visited: HashSet<State> = HashSet::new();

    queue.push_back(initial_state.clone());

    let mut start_time = std::time::Instant::now();
    while start_time.elapsed().as_millis() < 900 {
        // eprintln!("Queue: {:?}", queue.len());
        // if queue.len() > 30 {
        //     return None;
        // }
        match queue.pop_front() {
            None => return None,
            Some(current_state) => {
                if visited.contains(&current_state) {
                    // eprintln!("Already visited");
                    continue;
                }
                visited.insert(current_state.clone());

                // eprintln!("{}", visited.len());
                // eprintln!(
                //     "Current: {:?} - Actions {:?}",
                //     current_state.get_current_pos(),
                //     current_state.get_actions()
                // );
                if current_state.get_current_pos() == board.get_target() {
                    // eprintln!("Found the target");
                    // return Some(current_state);
                    return Some(current_state.clone());
                }

                // explore the neighbors
                for (offset, direction) in offset.iter() {
                    let new_idx = (current_state.get_current_pos() as i32 + offset) as usize;

                    match board.get_cell(new_idx) {
                        Cell::Switch(id) => {
                            let mut new_state = current_state.clone();
                            new_state.toggle_magnetic_field(*id);
                            new_state.add_actions(direction);
                            new_state.set_current_pos(new_idx);
                            queue.push_back(new_state);
                        }
                        Cell::MagneticField(id) => {
                            if current_state.is_magnetic_field_on(*id) {
                                // eprintln!("Magnetic field {} is on: {} -- Skipping", id, target);
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
                                        // eprintln!("Before: {:?}", current_state);
                                        new_state.add_actions(direction);
                                        new_state.set_current_pos(new_idx);
                                        // eprintln!("After: {:?}", new_state);
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
