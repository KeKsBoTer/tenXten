use priority_queue::PriorityQueue;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;
use std::{fmt, usize};

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

#[cfg(target_arch = "wasm32")]
extern crate console_error_panic_hook;

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

    /// This is the heuristic used for finding promising moves.
    /// The function counts how many moves are possible from each field and returns the sum.
    /// The larger the sum, the better is the move since it leaves more possible moves open.
    ///
    /// TODO: maybe only calculate the change of possible moves for a given move. This does not
    /// result in a better heuristic but could improve performance a bit.
    fn sum_moves(&self) -> usize {
        let mut sum = 0;
        for i in 0..self.size {
            for j in 0..self.size {
                if !self.occupied((j as usize, i as usize)) {
                    sum += MOVES
                        .iter()
                        .filter(|(ox, oy)| {
                            let x = j as i32 - ox;
                            let y = i as i32 - oy;
                            self.valid_and_not_occupied((x, y))
                        })
                        .count()
                }
            }
        }
        return sum;
    }

    fn possible_moves(&self, pos: Option<Move>) -> Vec<Move> {
        match pos {
            // at the beginning all moves are possible
            None => (1..self.size * self.size)
                .into_iter()
                .map(|i| ((i / self.size) as usize, (i % self.size) as usize))
                .collect(),
            Some(pos) => MOVES
                .iter()
                .filter_map(|(ox, oy)| {
                    let x = pos.0 as i32 - ox;
                    let y = pos.1 as i32 - oy;
                    if self.valid_and_not_occupied((x, y)) {
                        Some((x as usize, y as usize))
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }

    /// finds the location of a number within the board
    #[allow(dead_code)]
    fn num_pos(&self, n: usize) -> Option<Move> {
        if n > self.max_n {
            return None;
        }
        for i in 0..self.size {
            for j in 0..self.size {
                if self.field[i][j] == n {
                    return Some((j as usize, i as usize));
                }
            }
        }
        return None;
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
    fn push_moves(&self, queue: &mut PriorityQueue<State, MoveValue>) {
        for m in self.possible_moves() {
            let new_board = self.make_move(m);
            let priority = new_board.board.sum_moves();
            let depth = new_board.board.max_n;
            queue.push(new_board, (depth, priority));
        }
    }

    /// searches for all solutions and sends them through a channel
    #[cfg(not(target_arch = "wasm32"))]
    pub fn solve_all(&self, sender: Sender<State>) {
        let mut queue = PriorityQueue::<State, MoveValue>::new();

        self.push_moves(&mut queue);

        while !queue.is_empty() {
            let (state, _) = queue.pop().unwrap();

            if state.board.is_complete() {
                if !sender.send(state).is_ok() {
                    return;
                }
                continue;
            }

            state.push_moves(&mut queue);
        }
    }

    /// returns the first solution that is found
    #[cfg(not(target_arch = "wasm32"))]
    pub fn solve_one(&self) -> Option<State> {
        let (tx, rx): (Sender<State>, Receiver<State>) = channel();
        for m in self.possible_moves() {
            let sender = tx.clone();
            let state = self.make_move(m);
            //thread::spawn(move || state.solve_all(sender));
            state.solve_all(sender)
        }
        return rx.recv().ok();
    }

    #[cfg(target_arch = "wasm32")]
    pub fn solve_one(&self, start_idx: usize) -> Option<State> {
        let mut queue = PriorityQueue::<State, MoveValue>::new();

        let moves = self.possible_moves();
        let m = moves[start_idx % moves.len()];
        let new_board = self.make_move(m);
        let priority = new_board.board.sum_moves();
        let depth = new_board.board.max_n;
        queue.push(new_board, (depth, priority));

        while !queue.is_empty() {
            let (state, _) = queue.pop().unwrap();

            if state.board.is_complete() {
                return Some(state);
            }

            state.push_moves(&mut queue);
        }
        return None;
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
pub fn solve(js_object: &JsValue, start_idx: usize) -> Option<Array> {
    console_error_panic_hook::set_once();
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
    return state.solve_one(start_idx).and_then(|solution| {
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
