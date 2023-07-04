use std::fmt::Formatter;

use chess::{Color, Board, ChessMove};
use rand::Rng;

use super::chess_alg::{ChessAlgorithm, available_moves};

pub struct SingleLookaheadEngine {
    color: Color,
    eval: Box<dyn Fn(&Board, Color) -> f32>
}

impl std::fmt::Debug for SingleLookaheadEngine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SingleLookaheadEngine {{ color: {:?} }}", self.color)
    }
}

impl SingleLookaheadEngine {
    pub fn new<T: Fn(&Board, Color) -> f32 + 'static>(color: Color, eval: T) -> SingleLookaheadEngine {
        SingleLookaheadEngine {
            color,
            eval: Box::new(eval)
        }
    }
}

unsafe impl Send for SingleLookaheadEngine {}

impl ChessAlgorithm for SingleLookaheadEngine {
    fn get_move(&mut self, board: Board) -> ChessMove {
        let mut best_score = f32::NEG_INFINITY;
        let mut best_moves = Vec::new();

        for m in available_moves(&board) {
            let res = board.make_move_new(m);

            let score = (self.eval)(&res, self.color);

            if (score - best_score).abs() < 0.0001 {
                best_moves.push(m);
            } else if score > best_score {
                best_score = score;
                best_moves.clear();
                best_moves.push(m);
            }
        }

        let mut rng = rand::thread_rng();

        best_moves[rng.gen_range(0..best_moves.len())]
    }
}