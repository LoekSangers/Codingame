extern crate new_uttt;
extern crate rand;

use std::io;

use std::rc::Rc;

use new_uttt::game::game_state::GameState;
use new_uttt::masks::LOCAL_MOVES;
use new_uttt::{GameMove, Node};

fn main() {
    codingame();
}

#[inline]
fn codingame() {
    let mut rng = rand::thread_rng();
    let inputs = read_input();

    let mut game = GameState::default();
    let mut root: Rc<Node<GameMove>> = Rc::new(Node::new());
    let mut m: GameMove;

    match inputs.0.chars().next() {
        Some('-') => {
            game.inplace_move(&GameMove::from_coords(4, LOCAL_MOVES[4]));

            let mut unvisited_moves = root.unvisited_moves.borrow_mut();
            unvisited_moves.append(&mut game.all_moves());
            drop(unvisited_moves);

            Node::mcts(Rc::clone(&root), &mut game, 999999995, &mut rng);
            println!("4 4");
        },
        Some(_) => {
            let opponent_row = inputs.0.parse::<i8>().unwrap();
            let opponent_col = inputs.1.parse::<i8>().unwrap();

            game.inplace_move(&GameMove::from_coords(
                (opponent_col / 3 + (opponent_row / 3) * 3) as usize,
                LOCAL_MOVES[(opponent_col % 3 + (opponent_row % 3) * 3) as usize],
            ));

            let mut unvisited_moves = root.unvisited_moves.borrow_mut();
            unvisited_moves.append(&mut game.all_moves());
            drop(unvisited_moves);

            Node::mcts(Rc::clone(&root), &mut game.clone(), 999999997, &mut rng);

            let child = root.best_child();
            m = child.last_action.unwrap();
            root = child;

            game.inplace_move(&m);
            m.print();
        },
        _ => panic!(),
    }

    // game loop
    loop {
        let inputs = read_input();
        let opponent_row = inputs.0.trim().parse::<i8>().unwrap();
        let opponent_col = inputs.1.trim().parse::<i8>().unwrap();
        let opp_move = GameMove::from_coords(
            (opponent_col / 3 + (opponent_row / 3) * 3) as usize,
            LOCAL_MOVES[(opponent_col % 3 + (opponent_row % 3) * 3) as usize],
        );
        game.inplace_move(&opp_move);
        let next = Rc::clone(root
            .children
            .borrow()
            .iter()
            .find(|node| node.last_action == Some(opp_move))
            .unwrap());

        root = next;

        Node::mcts(Rc::clone(&root), &mut game, 99999900, &mut rng);

        let child = root.best_child();
        m = child.last_action.unwrap();
        root = child;
        game.inplace_move(&m);
        m.print();
    }
}

#[inline]
fn read_input() -> (String, String) {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(' ').collect::<Vec<_>>();
    let mut input_line_2 = String::new();
    io::stdin().read_line(&mut input_line_2).unwrap();
    let valid_action_count = input_line_2.trim().parse::<i32>().unwrap();
    for _ in 0..valid_action_count as usize {
        io::stdin().read_line(&mut input_line_2).unwrap();
    }

    (inputs[0].to_string(), inputs[1].to_string())
}
