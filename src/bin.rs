use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use std::{sync::mpsc, thread, time};
use structopt::StructOpt;

extern crate tenxten;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(short, long, help = "find all solutions (takes very long)")]
    find_all: bool,

    #[structopt(short, long, help = "shows additional information")]
    verbose: bool,

    #[structopt(short, long, help = "do not animate the solution")]
    no_animation: bool,

    #[structopt(
        short,
        long,
        default_value = "50",
        help = "delay (ms) between frames in the animation"
    )]
    animation_delay: u64,

    #[structopt(
        short,
        long,
        default_value = "10",
        help = "size (width and height) of the board"
    )]
    board_size: usize,

    #[structopt(name = "column", help = "start column")]
    x: usize,

    #[structopt(name = "row", help = "start row")]
    y: usize,
}

fn main() {
    let opt = Opt::from_args();

    if opt.x > opt.board_size || opt.y > opt.board_size {
        eprintln!(
            "start row ({}) and column ({}) must be within board size ({})",
            opt.y, opt.x, opt.board_size
        );
        return;
    }

    let mut state = tenxten::State::new(opt.board_size);
    state.make_move((opt.x - 1, opt.y - 1));

    if opt.verbose {
        println!("Initial board:\n{}", state.to_string());
    }

    let (tx, rx): (Sender<tenxten::State>, Receiver<tenxten::State>) = mpsc::channel();

    if opt.verbose {
        println!("searching for solution...");
    }

    state.solve_async(tx);

    if opt.verbose {
        println!("solution(s) found:");
    }
    for solution in rx.iter() {
        if opt.find_all || opt.no_animation {
            println!("{:}", &solution.to_string());
        }
        if !opt.find_all {
            solution.play_solution(Duration::from_millis(opt.animation_delay));
            return;
        }
    }
}
