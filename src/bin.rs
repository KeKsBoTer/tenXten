use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;

use structopt::StructOpt;

extern crate tenxten;

#[derive(StructOpt, Debug)]
#[structopt(name = "tenXten", about = "A cli for solving the 10x10 number game")]
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

    #[structopt(short, long, help = "write solution(s) to file")]
    output_file: Option<String>,

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

    let state = tenxten::State::new(opt.board_size).make_move((opt.x - 1, opt.y - 1));

    if opt.verbose {
        println!("Initial board:\n{}", state.to_string());
    }

    if opt.verbose {
        println!("searching for solution...");
    }

    let mut out_writer = match opt.output_file {
        Some(ref x) => {
            let path = Path::new(x);
            Box::new(File::create(&path).unwrap()) as Box<dyn Write>
        }
        None => Box::new(io::stdout()) as Box<dyn Write>,
    };

    if opt.find_all {
        let mut first = true;
        for (i, solution) in state.solve_all().enumerate() {
            if first && opt.verbose {
                first = false;
                println!("solutions found:");
            }
            out_writer
                .write_all(solution.to_string().as_bytes())
                .unwrap();
            out_writer.write_all("\n".as_bytes()).unwrap();
            if i % 10 == 0 {
                print!("\rsolutions found: {:}", i)
            }
        }
        if first && opt.verbose {
            println!("no solution found");
        }
    } else {
        if let Some(solution) = state.solve() {
            if opt.verbose {
                println!("solution found:");
            }
            if opt.no_animation || opt.output_file.is_some() {
                out_writer
                    .write_all(solution.to_string().as_bytes())
                    .unwrap();
                out_writer.write_all("\n".as_bytes()).unwrap();
            } else {
                solution.play(Duration::from_millis(opt.animation_delay));
            }
        } else if opt.verbose {
            println!("no solution found");
        }
    }
}
