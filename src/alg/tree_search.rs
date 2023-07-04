use std::fmt::Formatter;

use chess::{Color, Board, ChessMove, MoveGen};
use rand::Rng;

use super::chess_alg::{ChessAlgorithm, available_moves};

pub struct TreeSearchEngine {
    color: Color,
    eval: Box<dyn Fn(&Board, Color) -> f32 + Send>,
    depth: u32
}

impl std::fmt::Debug for TreeSearchEngine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TreeSearchEngine {{ color: {:?}, depth: {} }}", self.color, self.depth)
    }
}

impl TreeSearchEngine {
    pub fn new<T: 'static + Fn(&Board, Color) -> f32 + Send>(color: Color, eval: T, depth: u32) -> Self {
        Self {
            color,
            eval: Box::new(eval),
            depth
        }
    }

    fn alpha_beta_max(&self, board: Board, mut alpha: f32, beta: f32, depth: u32) -> f32 {
        if depth == 0 {
            return (self.eval)(&board, self.color);
        }

        for m in MoveGen::new_legal(&board) {
            let res = board.make_move_new(m);

            let score = self.alpha_beta_min(res, alpha, beta, depth - 1);

            if score >= beta {
                return beta;
            }

            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    fn alpha_beta_min(&self, board: Board, alpha: f32, mut beta: f32, depth: u32) -> f32 {
        if depth == 0 {
            return (self.eval)(&board, self.color);
        }

        for m in MoveGen::new_legal(&board) {
            let res = board.make_move_new(m);

            let score = self.alpha_beta_max(res, alpha, beta, depth - 1);

            if score <= alpha {
                return alpha;
            }

            if score < beta {
                beta = score;
            }
        }

        beta
    }
}

impl ChessAlgorithm for TreeSearchEngine {
    fn get_move(&mut self, board: Board) -> ChessMove {
        let moves = available_moves(&board);

        let mut best_score = f32::NEG_INFINITY;
        let mut best_moves = Vec::new();

        for m in moves {
            let res = board.make_move_new(m);

            let score = self.alpha_beta_min(res, f32::NEG_INFINITY, f32::INFINITY, self.depth);

            if (score - best_score).abs() < 0.0001 {
                best_moves.push(m);
            } else if score > best_score {
                best_score = score;
                best_moves.clear();
                best_moves.push(m);
            }
        }

        println!("Eval: {}", best_score);

        let mut rng = rand::thread_rng();

        best_moves[rng.gen_range(0..best_moves.len())]
    }
}