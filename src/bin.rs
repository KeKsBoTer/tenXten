use rand::seq::SliceRandom;
use std::{thread, time};
extern crate tenxten;

fn play_solution<const SIZE: usize>(state: &tenxten::State<SIZE>, delay: time::Duration) {
    let mut m_board = tenxten::State::<SIZE>::new();
    println!("{}", m_board.to_string());
    for i in 1..SIZE * SIZE + 1 {
        let pos = state.num_pos(i).unwrap();
        print!("{:}[{:}A", 27 as char, 2 * SIZE + 1);
        m_board.make_move(pos);
        println!("{}", m_board.to_string());
        thread::sleep(delay);
    }
}

fn main() {
    let starts = [
        (0, 0),
        (1, 1),
        (2, 2),
        (3, 3),
        (4, 4),
        (1, 0),
        (2, 1),
        (3, 2),
        (4, 3),
        (2, 0),
        (3, 1),
        (4, 2),
        (3, 0),
        (4, 1),
        (4, 0),
    ];

    let mut rng = rand::thread_rng();
    let start = starts.choose(&mut rng).unwrap();
    let mut state = tenxten::State::<10>::new();
    state.make_move(*start);

    println!("Initial board:\n{}", state.to_string());

    println!("Searching for a solution...");

    if let Some(solution) = state.find_solution() {
        play_solution(&solution, time::Duration::from_millis(100));
        println!("{:}", &solution.to_string())
    } else {
        println!("No solution found:");
    }
}
