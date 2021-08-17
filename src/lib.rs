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
    field: Box<[Box<[usize]>]>,
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
            field: (vec![(vec![0; size]).into_boxed_slice(); size]).into_boxed_slice(),
            max_n: 0,
            size: size,
        }
    }

    fn occupied(&self, m: Move) -> bool {
        self.field[m.1 as usize][m.0 as usize] != 0
    }

    fn add_number(&mut self, m: Move) {
        self.max_n += 1;
        self.field[m.1 as usize][m.0 as usize] = self.max_n;
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

    fn possible_moves(&self, pos: Option<Move>) -> Vec<Move> {
        match pos {
            // at the beginning all moves are possible
            None => (1..self.size * self.size)
                .into_iter()
                .filter_map(|i| Some(((i / self.size) as usize, (i % self.size) as usize)))
                .collect(),
            Some(pos) => MOVES
                .iter()
                .filter_map(|(ox, oy)| {
                    let x = pos.0 as i32 - ox;
                    let y = pos.1 as i32 - oy;
                    self.valid_and_not_occupied((x, y))
                        .then(|| (x as usize, y as usize))
                })
                .collect(),
        }
    }

    /// finds the location of a number within the board
    #[cfg(not(target_arch = "wasm32"))]
    fn num_pos(&self, n: usize) -> Option<Move> {
        if n > self.max_n {
            return None;
        }
        self.field.iter().enumerate().find_map(|(i, row)| {
            row.iter()
                .enumerate()
                .find_map(|(j, cell)| (*cell == n).then(|| (j, i)))
        })
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

    /// returns all possibles moves that can be done from the current state
    fn possible_moves(&self) -> Vec<Move> {
        self.board.possible_moves(self.pos)
    }

    pub fn make_move(&self, m: Move) -> State {
        let mut new_board = self.clone();
        new_board.board.add_number(m);
        new_board.pos = Some(m);
        return new_board;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn play(&self, delay: Duration) {
        let mut m_board = State::new(self.board.size);
        println!("{}", m_board.to_string());
        for i in 0..self.board.max_n {
            let pos = self.board.num_pos(i + 1).unwrap();
            print!("{:}[{:}A", 27 as char, 2 * self.board.size + 1);
            m_board = m_board.make_move(pos);
            println!("{}", m_board.to_string());
            thread::sleep(delay);
        }
    }

    /// pushes the next possible moves to a priority queue
    #[cfg(not(target_arch = "wasm32"))]
    fn push_moves(&self, queue: &mut PriorityQueue<State, MoveValue>) {
        queue.extend(self.possible_moves().into_iter().map(|m| {
            let new_board = self.make_move(m);
            // use Warnsdorff's rule as heuristic
            let priority = MOVES.len() - new_board.possible_moves().len();
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
        self.possible_moves()
            .into_iter()
            .cycle()
            .find_map(|open_move| {
                // makes the first move and then plays the game always making the "best" move
                // if the resulting board is a win, it is returned else the next start move is tried
                self.make_move(open_move)
                    .last()
                    .and_then(|solution| solution.board.is_complete().then(|| solution))
            })
    }
}

impl Iterator for State {
    type Item = State;
    fn next(&mut self) -> Option<Self::Item> {
        // all possible states after one move
        let mut boards: Vec<State> = self
            .possible_moves()
            .into_iter()
            .map(|m| self.make_move(m))
            .collect();

        let mut rng = rand::thread_rng();
        boards.shuffle(&mut rng);

        // find best move (the one with the fewest possible following moves)
        let best = boards.into_iter().min_by_key(|s| s.possible_moves().len());
        if let Some(best_state) = best.clone() {
            self.board = best_state.board;
            self.pos = best_state.pos;
        }
        best
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
        let possible_moves = self.possible_moves();

        let top = format!("╔══{:}═╗", "═╤══".repeat(self.board.size - 1));
        let divider = format!("\n╟──{:}─║\n", "─┼──".repeat(self.board.size - 1));
        let bottom = format!("╚══{:}═╝", "═╧══".repeat(self.board.size - 1));

        let content = self
            .board
            .field
            .iter()
            .enumerate()
            .map(|(i, row)| -> String {
                let r_f = row
                    .iter()
                    .enumerate()
                    .map(|(j, n)| {
                        if *n == 0 {
                            if possible_moves.contains(&(j as usize, i as usize)) {
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
    for i in 0..size {
        for j in 0..size {
            if field[i][j] > max_n {
                max_n = field[i][j];
                max_n_pos = Some((j, i));
            }
        }
    }
    let state = State {
        board: Board { field, max_n, size },
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
                    Number::from(solution.board.field[i][j] as u32).into(),
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
