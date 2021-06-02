use priority_queue::PriorityQueue;
use std::fmt;
use std::time::Duration;
use std::{sync::mpsc::Sender, thread, usize};

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
#[derive(Clone, Hash, Eq)]
struct Board {
    field: Box<[Box<[usize]>]>,
    max_n: usize,
    size: usize,
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.field == other.field
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

    fn num_pos(&self, n: usize) -> Option<Move> {
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
    fn possible_moves(&self) -> Vec<Move> {
        self.board.possible_moves(self.pos)
    }

    pub fn make_move(&self, m: Move) -> State {
        let mut new_board = self.clone();
        new_board.board.add_number(m);
        new_board.pos = Some(m);
        return new_board;
    }

    pub fn num_pos(&self, n: usize) -> Option<Move> {
        return self.board.num_pos(n);
    }

    pub fn play_solution(&self, delay: Duration) {
        let mut m_board = State::new(self.board.size);
        println!("{}", m_board.to_string());
        for i in 1..self.board.size * self.board.size + 1 {
            let pos = self.num_pos(i).unwrap();
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

    /// returns the first solution that is found
    pub fn solve_one(&self) -> Option<State> {
        let mut queue = PriorityQueue::<State, MoveValue>::new();

        self.push_moves(&mut queue);

        while !queue.is_empty() {
            let (state, _) = queue.pop().unwrap();

            if state.board.is_complete() {
                return Some(state);
            }

            state.push_moves(&mut queue);
        }

        return None;
    }

    /// searches for all solutions and sends them through a channel
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
