extern crate new_uttt;

use std::io;
use std::rc::Rc;
use std::time;

use new_uttt::game;
use new_uttt::mcts;
use new_uttt::cg_rand;

use game::game_action::Action;
use game::game_state::State;
use game::masks::LOCAL_MOVES;
use mcts::tree::MctsTree;
use cg_rand::Rng;
use mcts::traits::GameState;
use mcts::node::MctsNode;

fn main() {
    codingame();
}
fn perf_test() {
    let mut rng = Box::new(cg_rand::Rng::new());

    let mut game = State::default();
    let mcts = MctsTree::new(game);

    let mut action = Action::from_coords(4, LOCAL_MOVES[4]);

    let root_ref = mcts.root.borrow();
    let state = root_ref.state.perform_action_copy(&action);    
    let next = Rc::new(MctsNode::create_child(state, Rc::downgrade(&root_ref)));  
    drop(root_ref);   
    mcts.root.replace(next);

    let first_turn_begin = time::Instant::now();
    let first_duration = time::Duration::new(0, 999999995);
    mcts.expand_tree(first_turn_begin, first_duration, &mut rng, 81);
    println!("4 4");

    let duration = time::Duration::new(0, 99000000);
    // game loop
    loop {
        let begin = time::Instant::now();
        mcts.expand_tree(begin, duration, &mut rng, 5);

        let child = mcts.best_child();

        game = child.state.clone();
        action = game.last_action.unwrap();
        mcts.root.replace(child);
        

        action.print();
        if !game.playable(){
            
            println!("{:?}", game.outcome());
            return;
        }
        
        let opp = mcts.best_child();
        game = opp.state.clone();
        action = game.last_action.unwrap();
        mcts.root.replace(opp);

        action.print();
        if !game.playable(){
            
            println!("{:?}", game.outcome());
            return;
        }
    }
}

//#[inline]
fn codingame() {
    let mut rng: Box<Rng> = Box::new(cg_rand::Rng::new());
    let inputs = read_input();

    let game = State::default();
    let mcts = MctsTree::new(game);
    let mut action: Action;

    let first_turn_begin = time::Instant::now();
    let first_duration = time::Duration::new(0, 999999900);
    if inputs.0 < 0 {
        action = Action::from_coords(4, LOCAL_MOVES[4]);

        let root_ref = mcts.root.borrow();
        let state = root_ref.state.perform_action_copy(&action);    
        let next = Rc::new(MctsNode::create_child(state, Rc::downgrade(&root_ref)));  
        drop(root_ref);   
        mcts.root.replace(next);

        mcts.expand_tree(first_turn_begin, first_duration, &mut rng, 10);
        println!("4 4");
    } else {
        let opponent_row = inputs.0;
        let opponent_col = inputs.1;

        action = Action::from_coords(
            (opponent_col / 3 + (opponent_row / 3) * 3) as usize,
            LOCAL_MOVES[(opponent_col % 3 + (opponent_row % 3) * 3) as usize],
        );

        let root_ref = mcts.root.borrow();
        let state = root_ref.state.perform_action_copy(&action);    
        let next = Rc::new(MctsNode::create_child(state, Rc::downgrade(&root_ref)));  
        drop(root_ref);   
        mcts.root.replace(next);

        mcts.expand_tree(first_turn_begin, first_duration, &mut rng, 10);

        let child = mcts.root.borrow().best_child();
        action = child.state.last_action.unwrap();
        mcts.root.replace(child);

        action.print();
    }

    let turn_duration = time::Duration::new(0, 99999900);
    // game loop
    loop {
        let begin = time::Instant::now();
        let inputs = read_input();
        let opponent_row = inputs.0;
        let opponent_col = inputs.1;
        let opp_move = Action::from_coords(
            (opponent_col / 3 + (opponent_row / 3) * 3) as usize,
            LOCAL_MOVES[(opponent_col % 3 + (opponent_row % 3) * 3) as usize],
        );

        mcts.move_down(opp_move);
        mcts.expand_tree(begin, turn_duration, &mut rng, 10);

        let child = mcts.best_child();
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
        inputs[1].trim().parse::<i8>().unwrap()
    )
}
