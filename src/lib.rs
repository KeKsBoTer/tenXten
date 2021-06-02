use std::fmt;
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
struct Board<const SIZE: usize> {
    field: [[usize; SIZE]; SIZE],
    max_n: usize,
}

impl<const SIZE: usize> Board<SIZE> {
    fn occupied(&self, m: Move) -> bool {
        self.field[m.1 as usize][m.0 as usize] != 0
    }

    fn add_number(&mut self, m: Move) {
        self.max_n += 1;
        self.field[m.1 as usize][m.0 as usize] = self.max_n;
    }

    fn is_complete(&self) -> bool {
        self.max_n == SIZE * SIZE
    }

    fn valid_and_not_occupied(&self, (x, y): (i32, i32)) -> bool {
        x >= 0
            && y >= 0
            && (x as usize) < SIZE
            && (y as usize) < SIZE
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
        for i in 0..SIZE {
            for j in 0..SIZE {
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
            None => (1..SIZE * SIZE)
                .into_iter()
                .map(|i| ((i / SIZE) as usize, (i % SIZE) as usize))
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
        for i in 0..SIZE {
            for j in 0..SIZE {
                if self.field[i][j] == n {
                    return Some((j as usize, i as usize));
                }
            }
        }
        return None;
    }
}

impl<const SIZE: usize> Default for Board<SIZE> {
    fn default() -> Self {
        Board {
            field: [[0; SIZE]; SIZE],
            max_n: 0,
        }
    }
}

#[derive(Clone)]
pub struct State<const SIZE: usize> {
    board: Board<SIZE>,
    pos: Option<Move>,
}

impl<const SIZE: usize> State<SIZE> {
    pub fn new() -> Self {
        State {
            board: Board::default(),
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

    fn find_solutions(&self, solutions: &mut Vec<State<SIZE>>, find_any: bool) {
        let mut moves = self.possible_moves();

        if moves.len() == 0 {
            // game is done
            if self.board.is_complete() {
                solutions.push(self.clone());
            }
            return;
        }

        moves.sort_by_cached_key(|m| SIZE * SIZE * MOVES.len() - self.board.sum_moves(*m));

        for m in moves {
            let mut new_board = (*self).clone();
            new_board.make_move(m);
            new_board.find_solutions(solutions, find_any);
            if find_any && solutions.len() > 0 {
                return;
            }
        }
    }

    fn find_solutions_async(&self, sender: Sender<State<SIZE>>) {
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

        moves.sort_by_cached_key(|m| SIZE * SIZE * MOVES.len() - self.board.sum_moves(*m));

        for m in moves {
            let mut new_board = (*self).clone();
            new_board.make_move(m);
            new_board.find_solutions_async(sender.clone());
        }
    }

    pub fn find_solution(&self) -> Option<State<SIZE>> {
        let new_board = (*self).clone();
        let mut solutions = Vec::<State<SIZE>>::new();
        new_board.find_solutions(&mut solutions, true);
        return solutions.pop();
    }

    pub fn solve_async(&self, sender: Sender<State<SIZE>>) {
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
}

impl<const SIZE: usize> fmt::Display for State<SIZE> {
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

        let top = format!("╔══{:}═╗", "═╤══".repeat(SIZE - 1));
        let divider = format!("\n╟──{:}─║\n", "─┼──".repeat(SIZE - 1));
        let bottom = format!("╚══{:}═╝", "═╧══".repeat(SIZE - 1));

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
