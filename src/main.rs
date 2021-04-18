use std::{thread, time};
use rand::seq::SliceRandom;


type Move = (u8,u8);

type Path = Vec<Move>;
#[derive(Clone, Copy)]
struct Board{
    field:[[u8;10];10],
    max_n: u8
}

impl Board {
    fn occupied(&self,m:Move) -> bool{
        self.field[m.1 as usize][m.0 as usize] != 0
    }
    
    fn add_number(&mut self, m:Move){
        self.max_n+=1;
        self.field[m.1 as usize][m.0 as usize] = self.max_n;
    }

    fn is_complete(&self) -> bool{
        self.max_n == 100
    }

    fn sum_moves(&self) -> usize{
        let all_moves:Vec<(i8,i8)> = vec![
            (-3,0),
            (3,0),
            (0,-3),
            (0,3),
            (-2,-2),
            (-2,2),
            (2,-2),
            (2,2),
        ];
        let mut sum =0;
        for i in 0..10{
            for j in 0..10{
                if !self.occupied((j,i)){
                    sum += all_moves.iter().filter(|(ox,oy)|{
                        let x = j as i8 - ox;
                        let y = i as i8 - oy;
                        x >= 0 && y >=0 && x < 10 && y < 10 && !self.occupied((x as u8,y as u8))
                    }).count()
                }
            }
        }
        return sum;
    }
}

impl From<Path> for Board{
    fn from(path: Path) -> Self {
        let mut b = Board::default();
        for p in path{
            b.add_number(p)
        }
        b
    }
}

impl Default for Board {
    fn default() -> Self {
        Board{
            field:[[0;10];10],
            max_n:0,
        }
    }
}

#[derive(Clone)]
struct State {
    board:Board,
    pos: Option<Move>,
    path: Vec<Move>
}


impl State{
    fn new() -> Self{
        State{
            board:Board::default(),
            pos: None,
            path:vec![]
        }
    }
    fn possible_moves(&self) -> Vec<Move>{
        match self.pos{
            None => (1..100).into_iter().map(|i|(i/10 as u8,i%10 as u8)).collect(),
            Some(pos)=>{
                let all_moves:Vec<(i8,i8)> = vec![
                    (-3,0),
                    (3,0),
                    (0,-3),
                    (0,3),
                    (-2,-2),
                    (-2,2),
                    (2,-2),
                    (2,2),
                ];
                all_moves.iter().filter_map(|(ox,oy)|{
                    let x = pos.0 as i8 - ox;
                    let y = pos.1 as i8 - oy;
                    if x >= 0 && y >=0 && x < 10 && y < 10 && !self.board.occupied((x as u8,y as u8)){
                        Some((x as u8,y as u8))
                    }else{
                        None
                    }
                }).collect()
            }
        }
    }

    fn make_move(&mut self, m:Move){
        self.board.add_number(m);
        self.pos = Some(m);
        self.path.push(m);
    }

    fn find_solutions(&self) -> Option<State>{
        let mut moves =  self.possible_moves();

        if moves.len() == 0{
            // game is done 
            if self.board.is_complete(){
                return Some(self.clone());
            }else{
                return  None;
            }
        }

        let mut mm = moves.iter().map(|m|{
            let mut b = self.board.clone();
            b.add_number(*m);
            (b.sum_moves(),*m)
        }).collect::<Vec<(usize,Move)>>();
        mm.sort_by_key(|m|m.0);
        moves = mm.iter().rev().map(|m|m.1).collect::<Vec<Move>>();


        for m in moves{
            let mut new_board = (*self).clone();
            new_board.make_move(m);
            if let Some(solution) = new_board.find_solutions(){
                return  Some(solution);
            }
        }
        return  None;
    }

    fn solve(&self) -> Option<State>{
        let new_board = (*self).clone();
        new_board.find_solutions()
    }


    fn play_solution(&self,delay:time::Duration){
        let mut state = State::new();
        println!("{}",state.to_string());
        for m in self.path.iter(){
            print!("{:}[21A",27 as char);
            state.make_move(*m);
            println!("{}",state.to_string());
            thread::sleep(delay);
        }
    }
}

impl ToString for State{
    fn to_string(&self) -> String{
        let possible_moves = self.possible_moves();
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
        }).collect::<Vec<String>>().join("\n╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║\n");
        format!("╔═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╗\n{:}\n╚═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╝",content)
    }
}

impl ToString for Board{
    fn to_string(&self) -> String{
        let content = self.field.iter().map(|row|-> String {
            let r_f = row.iter().map(|n|{
                if *n==0{
                    String::from("   ")
                }else{
                    format!("{: >3}",*n)
                }
            }).collect::<Vec<String>>().join("│");
            format!("║{:}║",r_f)
        }).collect::<Vec<String>>().join("\n╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║\n");
        format!("╔═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╗\n{:}\n╚═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╝",content)
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
    let mut state = State::new();
    state.make_move(*start);

    println!("Initial board:\n{}",state.to_string());

    println!("Searching for a solution...");
    match state.solve(){
        Some(solution)=> {
            println!("Solution found:");
            print!("\x07");
            let delay = time::Duration::from_millis(100);
            solution.play_solution( delay)
        }
        None => {
            println!("No solution found:");
        }
    };
   
}
