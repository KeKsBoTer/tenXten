#[cfg(not(target_arch = "wasm32"))]
use priority_queue::PriorityQueue;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;
use std::{fmt, usize};

use rand::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc::{channel, Receiver, Sender};

#[cfg(not(target_arch = "wasm32"))]
use std::thread;

#[cfg(target_arch = "wasm32")]
use js_sys::{Array, Number};

#[cfg(target_arch = "wasm32")]
extern crate serde_json;
#[cfg(target_arch = "wasm32")]
extern crate wasm_bindgen;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate serde_derive;

const MOVES: [(i32, i32); 8] = [
    (-3, 0),
    (3, 0),
    (0, -3),
    (0, 3),
    (-2, -2),
    (-2, 2),
    (2, -2),
    (2, 2),
];

type Move = (usize, usize);

#[cfg(not(target_arch = "wasm32"))]
type MoveValue = (usize, usize);

#[cfg_attr(target_arch = "wasm32", derive(Serialize, Deserialize))]
#[derive(Clone, Hash, Eq)]
struct Board {
    field: Box<[usize]>,
    max_n: usize,
    size: usize,
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size && self.max_n == other.max_n && self.field == other.field
    }
}

impl Board {
    fn new(size: usize) -> Self {
        Board {
            field: (vec![0; size * size]).into_boxed_slice(),
            max_n: 0,
            size: size,
        }
    }

    fn occupied(&self, m: Move) -> bool {
        self.field[m.1 * self.size + m.0] != 0
    }

    fn add_number(&mut self, m: Move) {
        self.max_n += 1;
        self.field[m.1 * self.size + m.0] = self.max_n;
    }

    fn is_complete(&self) -> bool {
        self.max_n == self.size * self.size
    }

    fn valid_and_not_occupied(&self, (x, y): (i32, i32)) -> bool {
        x >= 0
            && y >= 0
            && (x as usize) < self.size
            && (y as usize) < self.size
            && !self.occupied((x as usize, y as usize))
    }

    fn possible_moves(&self, pos: Option<Move>) -> Box<dyn Iterator<Item = Move> + '_> {
        if pos.is_none() {
            Box::new((0..self.size * self.size).map(move |i| (i / self.size, i % self.size)))
        } else {
            let (px, py) = pos.unwrap();
            Box::new(MOVES.iter().filter_map(move |(ox, oy)| {
                let x = px as i32 - ox;
                let y = py as i32 - oy;
                self.valid_and_not_occupied((x, y))
                    .then(|| (x as usize, y as usize))
            }))
        }
    }
}

#[derive(Clone, Hash, Eq)]
pub struct State {
    board: Board,
    pos: Option<Move>,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board && self.pos == self.pos
    }
}

impl State {
    pub fn new(size: usize) -> Self {
        State {
            board: Board::new(size),
            pos: None,
        }
    }

    fn move_value(&mut self, m: Move) -> usize {
        let curr_pos = self.pos;
        self.apply_move(m);
        let p_moves = self.possible_moves().count();
        // undo move
        self.board.field[m.1 * self.board.size + m.0] = 0;
        self.board.max_n -= 1;
        self.pos = curr_pos;
        return p_moves;
    }

    // returns all possibles moves that can be done from the current state
    fn possible_moves(&self) -> Box<dyn Iterator<Item = Move> + '_> {
        self.board.possible_moves(self.pos)
    }

    pub fn make_move(&self, m: Move) -> State {
        let mut new_board = self.clone();
        new_board.board.add_number(m);
        new_board.pos = Some(m);
        return new_board;
    }

    pub fn apply_move(&mut self, m: Move) {
        self.board.add_number(m);
        self.pos = Some(m);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn play(&self, delay: Duration) {
        let mut m_board = State::new(self.board.size);
        println!("{}", m_board.to_string());
        let size = self.board.size;
        let mut moves = vec![None; size * size];
        for i in 0..size {
            for j in 0..size {
                let n = self.board.field[i * size + j];
                moves[n - 1] = Some((j, i));
            }
        }
        for m in moves {
            match m {
                Some(pos) => {
                    print!("{:}[{:}A", 27 as char, 2 * size + 1);
                    m_board = m_board.make_move(pos);
                    println!("{}", m_board.to_string());
                    thread::sleep(delay);
                }
                None => return,
            }
        }
    }

    /// pushes the next possible moves to a priority queue
    #[cfg(not(target_arch = "wasm32"))]
    fn push_moves(&self, queue: &mut PriorityQueue<State, MoveValue>) {
        queue.extend(self.possible_moves().into_iter().map(|m| {
            let new_board = self.make_move(m);
            // use Warnsdorff's rule as heuristic
            let priority = MOVES.len() - new_board.possible_moves().count();
            let depth = new_board.board.max_n;
            (new_board, (depth, priority))
        }));
    }

    /// searches for all solutions and sends them through a channel
    #[cfg(not(target_arch = "wasm32"))]
    pub fn find_solutions(&self, sender: Sender<State>) {
        let mut queue = PriorityQueue::<State, MoveValue>::new();

        self.push_moves(&mut queue);
        while let Some((state, _)) = queue.pop() {
            if state.board.is_complete() {
                if !sender.send(state).is_ok() {
                    return;
                }
                continue;
            }

            state.push_moves(&mut queue);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn solve_all(&self) -> impl Iterator<Item = State> {
        let (tx, rx): (Sender<State>, Receiver<State>) = channel();
        let s = Box::new(rx);
        for m in self.possible_moves() {
            let new_state = self.make_move(m);
            for m in new_state.possible_moves() {
                let sender = tx.clone();
                let state = new_state.make_move(m);
                thread::spawn(move || state.find_solutions(sender));
            }
        }
        return s.into_iter();
    }

    pub fn solve(&self) -> Option<State> {
        (1..5).find_map(|_| {
            self.possible_moves().find_map(|open_move| {
                // makes the first move and then plays the game always making the "best" move
                // if the resulting board is a win, it is returned else the next start move is tried
                let solution = self.make_move(open_move).take_best_steps();
                solution.board.is_complete().then(|| solution)
            })
        })
    }

    fn take_best_steps(&self) -> State {
        let mut tester = self.clone();
        let mut best = Vec::with_capacity(MOVES.len());
        let mut rng = rand::thread_rng();
        loop {
            let moves: Vec<Move> = tester.possible_moves().collect();
            if moves.len() == 0 {
                // we reached an end state
                break;
            }
            best.clear();
            let mut best_value = MOVES.len() + 1;
            for m in moves {
                let value = tester.move_value(m);
                if value < best_value {
                    best_value = value;
                    best.clear();
                    best.push(m);
                } else if value == best_value {
                    best.push(m);
                }
            }
            tester.apply_move(*best.choose(&mut rng).unwrap());
        }
        tester
    }
}

impl fmt::Display for State {
    /// Formats the board like this:
    /// ╔═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╗
    /// ║   │  2│   │   │  1│   │   │   │   │   ║
    /// ╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
    /// ║▒▒▒│   │   │ 12│   │   │ 11│   │   │ 10║
    /// ╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
    /// ║   │   │   │   │   │   │   │   │   │   ║
    /// ╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
    /// ║   │  3│   │   │   │▒▒▒│   │   │   │   ║
    /// ╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
    /// ║   │   │   │▒▒▒│   │   │   │   │   │  9║
    /// ╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
    /// ║   │   │   │   │   │   │   │   │   │   ║
    /// ╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
    /// ║   │  4│   │   │   │   │   │   │   │   ║
    /// ╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
    /// ║   │   │   │   │   │   │   │   │   │  8║
    /// ╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
    /// ║   │   │   │   │   │   │   │   │   │   ║
    /// ╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
    /// ║   │  5│   │   │  6│   │   │  7│   │   ║
    /// ╚═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╝
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let possible_moves: Vec<Move> = self.possible_moves().collect();

        let top = format!("╔══{:}═╗", "═╤══".repeat(self.board.size - 1));
        let divider = format!("\n╟──{:}─║\n", "─┼──".repeat(self.board.size - 1));
        let bottom = format!("╚══{:}═╝", "═╧══".repeat(self.board.size - 1));

        let content = self
            .board
            .field
            .chunks(self.board.size)
            .enumerate()
            .map(|(i, row)| -> String {
                let r_f = row
                    .iter()
                    .enumerate()
                    .map(|(j, n)| {
                        if *n == 0 {
                            if possible_moves.iter().any(|m| m.0 == j && m.1 == i) {
                                String::from("▒▒▒")
                            } else {
                                String::from("   ")
                            }
                        } else {
                            format!("{: >3}", *n)
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("│");
                format!("║{:}║", r_f)
            })
            .collect::<Vec<String>>()
            .join(&divider);
        write!(f, "{:}\n{:}\n{:}", top, content, bottom)
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn solve(js_object: &JsValue) -> Option<Array> {
    let field: Box<[Box<[usize]>]> = js_object.into_serde().unwrap();
    let size = field.len();
    let mut max_n = 0;
    let mut max_n_pos = None;
    let mut flat_field = vec![0; size * size];
    for i in 0..size {
        for j in 0..size {
            flat_field[i * size + j] = field[i][j];
            if field[i][j] > max_n {
                max_n = field[i][j];
                max_n_pos = Some((j, i));
            }
        }
    }
    let state = State {
        board: Board {
            field: flat_field.into_boxed_slice(),
            max_n,
            size,
        },
        pos: max_n_pos,
    };
    // convert to 2d js array
    return state.solve().and_then(|solution| {
        let size = solution.board.size;
        let a = Array::new_with_length(size as u32);
        for i in 0..size {
            let b = Array::new_with_length(size as u32);
            for j in 0..size {
                b.set(
                    j as u32,
                    Number::from(solution.board.field[i * size + j] as u32).into(),
                );
            }
            a.set(i as u32, b.into());
        }
        Some(a)
    });
}
#[cfg(test)]
mod tests {
    use crate::State;

    // test the solver for a given board size
    // it tests all possible starts and checks if a solution can be found
    fn test_for_baord_size(size: usize) {
        for i in 0..size {
            for j in 0..size {
                let start_state = State::new(size).make_move((j, i));
                let solution = start_state.solve();
                assert!(
                    solution.is_some(),
                    "cannot solve board with start {} {}",
                    j + 1,
                    i + 1
                )
            }
        }
    }

    // create a macro that runs the test for a given board size and size in the test name
    macro_rules! solver_test {
        ($($size:expr),*)  => {
            $(paste::item! {
                #[test]
                fn [< solve_board_ $size x $size>]() {
                    test_for_baord_size($size)
                }
            })*
        }
    }
    solver_test!(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20);
}
