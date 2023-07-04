use std::{thread, time::Duration};

use chess::{Board, ChessMove, MoveGen};
use rand::Rng;

use crate::util::move_to_SAN;

pub fn available_moves(board: &Board) -> Vec<ChessMove> {
    MoveGen::new_legal(&board).collect::<Vec<ChessMove>>()
}

pub trait ChessAlgorithm : std::fmt::Debug + Send {
    fn get_move(&mut self, board: Board) -> ChessMove;
    fn do_move(&mut self, board: Board, chess_move: ChessMove) {
        
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RandomChessAlgorithm;

unsafe impl Send for RandomChessAlgorithm {}

impl ChessAlgorithm for RandomChessAlgorithm {
    fn get_move(&mut self, board: Board) -> ChessMove {
        let moves = available_moves(&board);

        let mut rng = rand::thread_rng();

        moves[rng.gen_range(0..moves.len())]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FirstMoveAlgorithm;

unsafe impl Send for FirstMoveAlgorithm {}

impl ChessAlgorithm for FirstMoveAlgorithm {
    fn get_move(&mut self, board: Board) -> ChessMove {
        let moves = available_moves(&board);

        let white_key = |m: &&ChessMove| (
            m.get_source().get_rank().to_index(),
            m.get_source().get_file().to_index(),

            m.get_dest().get_rank().to_index(),
            m.get_dest().get_file().to_index(),

            m.get_promotion()
        );

        let black_key = |m: &&ChessMove| (
            7 - m.get_source().get_rank().to_index(),
            m.get_source().get_file().to_index(),

            7 - m.get_dest().get_rank().to_index(),
            m.get_dest().get_file().to_index(),

            m.get_promotion()
        );

        let key = if board.side_to_move() == chess::Color::White {
            white_key
        } else {
            black_key
        };

        *moves.iter().min_by_key(key).unwrap()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AlphabeticalChessAlgorithm;

unsafe impl Send for AlphabeticalChessAlgorithm {}

impl ChessAlgorithm for AlphabeticalChessAlgorithm {
    fn get_move(&mut self, board: Board) -> ChessMove {
        let moves = available_moves(&board);

        let key = |m: &&ChessMove| (
            move_to_SAN(&board, **m).to_ascii_lowercase()
        );

        *moves.iter().min_by_key(key).unwrap()
    }
}