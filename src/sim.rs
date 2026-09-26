//! Simulateur fidèle au moteur (eulerscheZahl/Bender4 : Robot, Box, Interpreter,
//! Referee). Il tourne sur la carte NON simplifiée : un cul-de-sac supprimé par
//! `Board::simplify` est un mur pour le solveur mais pas pour le vrai jeu.
//!
//! Règles reprises du moteur :
//! - un coup contre un mur (ou une ball non poussable) ne bouge pas Bender ;
//! - une ball est poussable si la case derrière est libre (ni mur ni ball) ;
//!   un champ magnétique ne bloque PAS une ball ;
//! - entrer sur un switch le toggle ; une ball qui atterrit sur un switch aussi ;
//! - après chaque coup : Bender sur un champ actif => perdu ; ball sur Fry =>
//!   perdu ; Bender sur Fry => gagné ;
//! - un chiffre est un coup à vide qui consomme un tour ; chaque retour de
//!   fonction consomme aussi un tour ; le jeu s'arrête après 1000 tours.

use crate::board::{Board, Cell, State};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Outcome {
    Continue,
    Win,
    Lose,
}

/// Le moteur coupe à 1000 tours ; marge de 10 pour l'éventuel tour de décalage.
const MAX_TURNS: usize = 990;

pub struct Sim<'a> {
    board: &'a Board,
    state: State,
}

impl<'a> Sim<'a> {
    pub fn new(board: &'a Board, state: &State) -> Sim<'a> {
        Sim {
            board,
            state: state.clone(),
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    /// Exécute un caractère (U/D/L/R ; tout autre caractère = coup à vide).
    pub fn step(&mut self, c: u8) -> Outcome {
        let pos = self.state.get_current_pos();
        let next = match c {
            b'L' => pos - 1,
            b'R' => pos + 1,
            b'U' => pos - 21,
            b'D' => pos + 21,
            _ => pos,
        };

        if next != pos {
            // ball + (ball - moi), en signé : peut sortir de la grille près du bord
            let bt = 2 * next as i32 - pos as i32;
            let ball_target = bt as usize;
            let blocked = match self.board.get_cell(next) {
                Cell::Wall => true,
                _ => {
                    self.state.is_garbage_ball(next)
                        && (!(0..441).contains(&bt)
                            || matches!(self.board.get_cell(ball_target), Cell::Wall)
                            || self.state.is_garbage_ball(ball_target))
                }
            };
            if !blocked {
                if let Cell::Switch(id) = self.board.get_cell(next) {
                    self.state.toggle_magnetic_field(*id);
                }
                if self.state.is_garbage_ball(next) {
                    self.state.move_ball(next, ball_target);
                    if let Cell::Switch(id) = self.board.get_cell(ball_target) {
                        self.state.toggle_magnetic_field(*id);
                    }
                }
                self.state.set_current_pos(next);
            }
        }

        self.outcome()
    }

    fn outcome(&self) -> Outcome {
        let pos = self.state.get_current_pos();
        if let Cell::MagneticField(id) = self.board.get_cell(pos) {
            if self.state.is_magnetic_field_on(*id) {
                return Outcome::Lose;
            }
        }
        let target = self.board.get_target();
        if self.state.is_garbage_ball(target) {
            return Outcome::Lose;
        }
        if pos == target {
            return Outcome::Win;
        }
        Outcome::Continue
    }
}

/// Exécute un programme complet ("core;f1;f2;...") comme l'interpréteur du jeu.
/// Retourne true seulement si Bender atteint Fry sans mourir dans la limite de tours.
pub fn wins(board: &Board, state: &State, program: &str) -> bool {
    let functions: Vec<&[u8]> = program.split(';').map(|f| f.as_bytes()).collect();
    let mut sim = Sim::new(board, state);
    if sim.outcome() != Outcome::Continue {
        return sim.outcome() == Outcome::Win;
    }

    let mut stack: Vec<(usize, usize)> = vec![(0, 0)]; // (fonction, index)
    for _ in 0..MAX_TURNS {
        let Some(&(f, i)) = stack.last() else {
            return false; // plus de commandes : "Invalid path"
        };
        if i >= functions[f].len() {
            stack.pop();
            continue;
        }
        stack.last_mut().unwrap().1 += 1;
        let c = functions[f][i];
        match sim.step(c) {
            Outcome::Win => return true,
            Outcome::Lose => return false,
            Outcome::Continue => {}
        }
        if (b'1'..=b'9').contains(&c) {
            let idx = (c - b'0') as usize;
            if idx >= functions.len() {
                return false;
            }
            stack.push((idx, 0));
        }
    }
    false
}

/// Pour chaque coup de `path`, indique si le REJOUER juste après serait un coup
/// perdu (état inchangé : mur ou ball bloquée). Ces coups en trop sont gratuits :
/// on peut allonger la ligne droite sans changer le résultat.
pub fn repeat_is_free(board: &Board, state: &State, path: &str) -> Vec<bool> {
    let mut sim = Sim::new(board, state);
    let mut out = Vec::with_capacity(path.len());
    for &c in path.as_bytes() {
        sim.step(c);
        let before = sim.state().clone();
        let mut probe = Sim::new(board, &before);
        probe.step(c);
        out.push(*probe.state() == before);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Couloir horizontal fermé : Bender en (1,1), Fry en (3,1).
    fn corridor() -> (Board, State) {
        let mut b = Board::new();
        for x in 1..=3 {
            b.set_cell(x, 1, Cell::Empty);
        }
        b.set_start(1, 1);
        b.set_target(3, 1);
        let mut s = State::new(0);
        s.set_current_pos(b.get_start());
        (b, s)
    }

    #[test]
    fn plain_path_and_overshoot() {
        let (b, s) = corridor();
        assert!(wins(&b, &s, "RR"));
        assert!(!wins(&b, &s, "R"));
        assert!(wins(&b, &s, "UUDDLLRR")); // coups perdus contre les murs
    }

    #[test]
    fn recursion_stops_at_fry() {
        let (b, s) = corridor();
        assert!(wins(&b, &s, "1;R1"));
        assert!(!wins(&b, &s, "1;L1")); // boucle infinie : coupé à 1000 tours
        assert!(!wins(&b, &s, "2;R")); // appel d'une fonction inexistante
    }

    #[test]
    fn free_repeats() {
        let (b, s) = corridor();
        let free = repeat_is_free(&b, &s, "R");
        assert_eq!(free, vec![false]); // Bender est en (2,1), pas contre un mur
        let free = repeat_is_free(&b, &s, "LLR");
        assert_eq!(free, vec![true, true, false]);
    }
}
