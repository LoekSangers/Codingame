extern crate new_uttt;
extern crate rand;

use std::io;

use new_uttt::game;
use new_uttt::mcts;

use game::game_action::Action;
use game::game_state::State;
use game::masks::LOCAL_MOVES;
use mcts::tree::MctsTree;
// use mcts::traits::GameState;

fn main() {
    codingame();
}
// fn perf_test() {
//     let mut rng = rand::thread_rng();

//     let mut game = State::default();
//     let mcts = MctsTree::new(game);

//     let mut action = Action::from_coords(4, LOCAL_MOVES[4]);

//     mcts.move_down(action);
//     mcts.expand_tree(999999990, &mut rng);
//     println!("4 4");

//     // game loop
//     loop {
//         mcts.expand_tree(99000000, &mut rng);

//         let child = mcts.root.borrow().best_child();
//         game = child.state.clone();
//         action = game.last_action.unwrap();
//         mcts.root.replace(child);

//         action.print();
//         if !game.playable(){
            
//             println!("{:?}", game.outcome());
//             return;
//         }
//     }
// }

//#[inline]
fn codingame() {
    let mut rng = rand::thread_rng();
    let inputs = read_input();

    let game = State::default();
    let mcts = MctsTree::new(game);
    let mut action: Action;

    if inputs.0 < 0 {
        action = Action::from_coords(4, LOCAL_MOVES[4]);

        mcts.move_down(action);
        mcts.expand_tree(999999995, &mut rng);
        println!("4 4");
    } else {
        let opponent_row = inputs.0;
        let opponent_col = inputs.1;

        action = Action::from_coords(
            (opponent_col / 3 + (opponent_row / 3) * 3) as usize,
            LOCAL_MOVES[(opponent_col % 3 + (opponent_row % 3) * 3) as usize],
        );

        mcts.move_down(action);
        mcts.expand_tree(999999995, &mut rng);

        let child = mcts.root.borrow().best_child();
        action = child.state.last_action.unwrap();
        mcts.root.replace(child);

        action.print();
    }

    // game loop
    loop {
        let inputs = read_input();
        let opponent_row = inputs.0;
        let opponent_col = inputs.1;
        let opp_move = Action::from_coords(
            (opponent_col / 3 + (opponent_row / 3) * 3) as usize,
            LOCAL_MOVES[(opponent_col % 3 + (opponent_row % 3) * 3) as usize],
        );

        mcts.move_down(opp_move);
        mcts.expand_tree(99999995, &mut rng);

        let child = mcts.root.borrow().best_child();
        action = child.state.last_action.unwrap();
        mcts.root.replace(child);

        action.print();
    }
}

//#[inline]
fn read_input() -> (i8, i8) {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split_ascii_whitespace().collect::<Vec<_>>();
    let mut input_line_2 = String::new();
    io::stdin().read_line(&mut input_line_2).unwrap();
    let valid_action_count = input_line_2.trim().parse::<i32>().unwrap();
    for _ in 0..valid_action_count as usize {
        io::stdin().read_line(&mut input_line_2).unwrap();
    }

    (
        inputs[0].trim().parse::<i8>().unwrap(),
        inputs[1].trim().parse::<i8>().unwrap(),
    )
}
