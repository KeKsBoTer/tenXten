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
#[derive(Clone)]
struct Board {
    field: Box<[Box<[usize]>]>,
    max_n: usize,
    size: usize,
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
    fn sum_moves(&self, pos: Move) -> usize {
        let occupied = |m: Move| m == pos || self.occupied(m);
        let mut sum = 0;
        for i in 0..self.size {
            for j in 0..self.size {
                if !occupied((j as usize, i as usize)) {
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

#[derive(Clone)]
pub struct State {
    board: Board,
    pos: Option<Move>,
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

    pub fn make_move(&mut self, m: Move) {
        self.board.add_number(m);
        self.pos = Some(m);
    }

    fn find_solutions(&self, solutions: &mut Vec<State>, find_any: bool) {
        let mut moves = self.possible_moves();

        if moves.len() == 0 {
            // game is done
            if self.board.is_complete() {
                solutions.push(self.clone());
            }
            return;
        }

        moves.sort_by_cached_key(|m| {
            self.board.size * self.board.size * MOVES.len() - self.board.sum_moves(*m)
        });

        for m in moves {
            let mut new_board = (*self).clone();
            new_board.make_move(m);
            new_board.find_solutions(solutions, find_any);
            if find_any && solutions.len() > 0 {
                return;
            }
        }
    }

    fn find_solutions_async(&self, sender: Sender<State>) {
        let mut moves = self.possible_moves();

        if moves.len() == 0 {
            // game is done
            if self.board.is_complete() {
                if !sender.send(self.clone()).is_ok() {
                    return;
                }
            }
            return;
        }

        moves.sort_by_cached_key(|m| {
            self.board.size * self.board.size * MOVES.len() - self.board.sum_moves(*m)
        });

        for m in moves {
            let mut new_board = (*self).clone();
            new_board.make_move(m);
            new_board.find_solutions_async(sender.clone());
        }
    }

    pub fn find_solution(&self) -> Option<State> {
        let new_board = (*self).clone();
        let mut solutions = Vec::<State>::new();
        new_board.find_solutions(&mut solutions, true);
        return solutions.pop();
    }

    pub fn solve_async(&self, sender: Sender<State>) {
        let moves = self.possible_moves();
        let mut children = Vec::new();
        for m in moves {
            let mut state = self.clone();
            let tx = sender.clone();
            state.make_move(m);
            let child = thread::spawn(move || {
                state.find_solutions_async(tx);
            });
            children.push(child);
        }
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
            m_board.make_move(pos);
            println!("{}", m_board.to_string());
            thread::sleep(delay);
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
