use std::fmt::{Debug, Formatter, Result};
use std::hash::{Hash, Hasher};

/// Contenu statique d'une case. Les balls ne sont pas ici : elles bougent, donc
/// elles vivent dans `State`. L'usize de Switch/MagneticField est l'id de la paire.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
    Wall,
    Switch(usize),
    MagneticField(usize),
    Empty,
}

/// État mutable d'une recherche : position, balls, champs actifs et chemin parcouru.
/// Positions = index plat dans la grille 21x21 (y * 21 + x).
/// Hash/Eq ignorent `actions` et `move_balls` : seul (pos, balls, champs) compte
/// pour dédupliquer les états visités.
pub struct State {
    current_pos: usize,
    garbage_balls: Vec<usize>, // positions des balls
    move_balls: Vec<u8>,       // nb de poussées par ball (même indexation)
    actions: String,           // chemin parcouru (U/D/L/R)
    magnetic_fields: u16,      // bitmask : bit i = champ i actif (létal)
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

    /// Inverse l'état du champ `idx` (ON <-> OFF).
    pub fn toggle_magnetic_field(&mut self, idx: usize) {
        self.magnetic_fields ^= 1 << idx;
    }

    pub fn is_magnetic_field_on(&self, idx: usize) -> bool {
        self.magnetic_fields & (1 << idx) != 0
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


    /// Déplace la ball située en `from_idx` vers `to_idx` (no-op si pas de ball).
    pub fn move_ball(&mut self, from_idx: usize, to_idx: usize) {
        if let Some(i) = self.get_ball_id(from_idx) {
            self.garbage_balls[i] = to_idx;
            self.move_balls[i] += 1;
        }
    }

    pub fn get_ball_id(&self, idx: usize) -> Option<usize> {
        self.garbage_balls.iter().position(|&x| x == idx)
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

/// Grille fixe 21x21 (taille max du jeu) + points d'intérêt.
/// Les cases hors de la zone lue restent des murs.
#[derive(Clone)]
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

    /// Pose une case en (x, y) et mémorise la position des switchs/champs par id.
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

    /// Debug : affiche la grille sur stderr (# mur, . vide, S switch, M champ, + ball).
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
}
