use std::{thread, time, usize};
use rand::seq::SliceRandom;

const MOVES:[(i8,i8);8] = [
    (-3,0),
    (3,0),
    (0,-3),
    (0,3),
    (-2,-2),
    (-2,2),
    (2,-2),
    (2,2),
];

type Move = (u8,u8);
#[derive(Clone, Copy)]
struct Board<const SIZE: usize>{
    field:[[usize;SIZE];SIZE],
    max_n: usize
}

impl<const SIZE: usize> Board<SIZE>{
    fn occupied(&self,m:Move) -> bool{
        self.field[m.1 as usize][m.0 as usize] != 0
    }
    
    fn add_number(&mut self, m:Move){
        self.max_n+=1;
        self.field[m.1 as usize][m.0 as usize] = self.max_n;
    }

    fn is_complete(&self) -> bool{
        self.max_n == SIZE*SIZE
    }

    fn valid_and_not_occupied(&self,(x,y):(i8,i8)) -> bool{
        x >= 0 && y >=0 && (x as usize) < SIZE && (y as usize) < SIZE && !self.occupied((x as u8, y as u8))
    }

    fn sum_moves(&self,pos:Move) -> usize{
        let occupied = |m:Move| m == pos || self.occupied(m);
        let mut sum =0;
        for i in 0..SIZE{
            for j in 0..SIZE{
                if !occupied((j as u8,i as u8)){
                    sum += MOVES.iter().filter(|(ox,oy)|{
                        let x = j as i8 - ox;
                        let y = i as i8 - oy;
                        self.valid_and_not_occupied((x,y))
                    }).count()
                }
            }
        }
        return sum;
    }


    fn possible_moves(&self, pos: Option<Move>) -> Vec<Move>{
        match pos{
            None => (1..SIZE*SIZE).into_iter().map(|i|((i/SIZE) as u8,(i%SIZE) as u8)).collect(),
            Some(pos)=>{
                MOVES.iter().filter_map(|(ox,oy)|{
                    let x = pos.0 as i8 - ox;
                    let y = pos.1 as i8 - oy;
                    if self.valid_and_not_occupied((x,y)){
                        Some((x as u8,y as u8))
                    }else{
                        None
                    }
                }).collect()
            }
        }
    }

    fn num_pos(&self, n:usize) -> Option<Move>{
        for i in 0..SIZE {
            for j in 0..SIZE {
                if self.field[i][j] == n{
                    return Some((j as u8,i as u8));
                }
            }
        }
        return None;
    }
}

impl<const SIZE:usize> Default for Board<SIZE> {
    fn default() -> Self {
        Board{
            field:[[0;SIZE];SIZE],
            max_n:0,
        }
    }
}

#[derive(Clone)]
struct State<const SIZE: usize> {
    board:Board<SIZE>,
    pos: Option<Move>
}


impl<const SIZE: usize> State<SIZE>{
    fn new() -> Self{
        State{
            board:Board::default(),
            pos: None
        }
    }
    fn possible_moves(&self) -> Vec<Move>{
        self.board.possible_moves(self.pos)
    }

    fn make_move(&mut self, m:Move){
        self.board.add_number(m);
        self.pos = Some(m);
    }

    fn find_solutions(&self,solutions:&mut Vec<State<SIZE>>,find_any:bool){
        let mut moves =  self.possible_moves();

        if moves.len() == 0{
            // game is done 
            if self.board.is_complete(){
                solutions.push(self.clone());
            }
            return;
        }

        moves.sort_by_cached_key(|m|{
            - (self.board.sum_moves(*m) as i32)
        });

        for m in moves{
            let mut new_board = (*self).clone();
            new_board.make_move(m);
            new_board.find_solutions(solutions,find_any);
            if find_any && solutions.len() > 0 {
                return;
            }
        }
    }

    fn solve(&self, find_any:bool) -> Vec<State<SIZE>>{
        let new_board = (*self).clone();
        let mut solutions = Vec::<State<SIZE>>::new();
        new_board.find_solutions(&mut solutions,find_any);
        return  solutions;
    }


    fn play_solution(&self,delay:time::Duration){
        let mut state = State::<SIZE>::new();
        println!("{}",state.to_string());
        for i in 1..SIZE*SIZE+1{
            let pos = self.board.num_pos(i).unwrap();
            print!("{:}[{:}A",27 as char,2*SIZE+1);
            state.make_move(pos);
            println!("{}",state.to_string());
            thread::sleep(delay);
        }
    }
}

impl<const SIZE: usize> ToString for State<SIZE>{
    fn to_string(&self) -> String{
        let possible_moves = self.possible_moves();
        
        let top = format!("╔══{:}═╗","═╤══".repeat(SIZE-1));
        let divider = format!("\n╟──{:}─║\n","─┼──".repeat(SIZE-1));
        let bottom = format!("╚══{:}═╝","═╧══".repeat(SIZE-1));

        let content = self.board.field.iter().enumerate().map(|(i,row)|-> String {
            let r_f = row.iter().enumerate().map(|(j,n)|{
                if *n==0{
                    if possible_moves.contains(&(j as u8, i as u8)){
                        String::from("▒▒▒")
                    }else{
                        String::from("   ")
                    }
                }else{
                    format!("{: >3}",*n)
                }
            }).collect::<Vec<String>>().join("│");
            format!("║{:}║",r_f)
        }).collect::<Vec<String>>().join(&divider);
        format!("{:}\n{:}\n{:}",top,content,bottom)
    }
}

fn main() {

    let starts = [
        (0,0), (1,1), (2,2), (3,3), (4,4),
        (1,0), (2,1), (3,2), (4,3),
        (2,0), (3,1), (4,2),
        (3,0), (4,1),
        (4,0),
    ];

    let mut rng = rand::thread_rng();
    let start = starts.choose(&mut rng).unwrap();
    let mut state = State::<10>::new();
    state.make_move(*start);

    println!("Initial board:\n{}",state.to_string());

    println!("Searching for a solution...");
    let solutions = state.solve(false);
    if solutions.is_empty(){
        println!("No solution found:");
    }else{
        println!("{:} solutions found:",solutions.len());
        for s in solutions{
            println!("{:}",s.to_string())
        }
    }
   
}
