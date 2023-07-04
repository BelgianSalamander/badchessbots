use chess::{Board, Color, ALL_SQUARES, Square, BoardStatus};

use super::chess_alg::available_moves;

fn square_color(square: chess::Square) -> Color {
    let rank = square.get_rank().to_index();
    let file = square.get_file().to_index();

    if (rank + file) % 2 == 0 {
        Color::Black
    } else {
        Color::White
    }
}

fn opposite(color: Color) -> Color {
    match color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    }
}

fn value_of_piece(piece: chess::Piece) -> f32 {
    match piece {
        chess::Piece::Pawn => 1.0,
        chess::Piece::Knight => 3.0,
        chess::Piece::Bishop => 3.0,
        chess::Piece::Rook => 5.0,
        chess::Piece::Queen => 9.0,
        chess::Piece::King => 0.0,
    }
}

fn chebyshev(a: Square, b: Square) -> i32 {
    let rank_diff = ((a.get_rank().to_index() as i32) - (b.get_rank().to_index() as i32)).abs();
    let file_diff = ((a.get_file().to_index() as i32) - (b.get_file().to_index() as i32)).abs();

    rank_diff.max(file_diff)
}

pub fn eval_matching_colors(board: &Board, color: Color) -> f32 {
    let mut score = 0.0;

    for square in ALL_SQUARES {
        if Some(color) == board.color_on(square) {
            if color == square_color(square) {
                score += 1.0;
            } else {
                score -= 1.0;
            }
        }
    }

    score
}

pub fn eval_opposite_colors(board: &Board, color: Color) -> f32 {
    let mut score = 0.0;

    for square in ALL_SQUARES {
        if Some(color) == board.color_on(square) {
            if color != square_color(square) {
                score += 1.0;
            } else {
                score -= 1.0;
            }
        }
    }

    score
}

pub fn eval_huddle(board: &Board, color: Color) -> f32 {
    let mut dist = 0.0;

    let target = board.king_square(color);

    for square in ALL_SQUARES {
        if Some(color) == board.color_on(square) {
            dist += chebyshev(square, target) as f32;
        }
    }

    -dist
}

pub fn eval_swarm(board: &Board, color: Color) -> f32 {
    let mut dist = 0.0;

    let target = board.king_square(opposite(color));

    for square in ALL_SQUARES {
        if Some(color) == board.color_on(square) {
            dist += chebyshev(square, target) as f32;
        }
    }

    -dist
}

pub fn eval_pacifist(board: &Board, color: Color) -> f32 {
    if board.status() == BoardStatus::Checkmate {
        return -10e20;
    } else if board.checkers().0 != 0 {
        return -10e10;
    } else {
        let mut opposite_value = 0.0;

        for square in ALL_SQUARES {
            if Some(opposite(color)) == board.color_on(square) {
                opposite_value += value_of_piece(board.piece_on(square).unwrap());
            }
        }

        return opposite_value;
    }
}

pub fn eval_generous(board: &Board, color: Color) -> f32 {
    if board.side_to_move() == color {
        panic!("Generous evaluator should only be used for the opponent!");
    }

    let mut score = 0.0;

    for m in available_moves(board) {
        if let Some(capture) = board.piece_on(m.get_dest()) {
            score += value_of_piece(capture);
        }
    }

    score as f32
}

pub fn eval_insist_2(board: &Board, color: Color) -> f32 {
    if board.side_to_move() == color {
        panic!("Insist 2 evaluator should only be used for the opponent!");
    }

    let status = board.status();

    if status == BoardStatus::Checkmate {
        return -10e20;
    } else if status == BoardStatus::Stalemate {
        return -10e10;
    }

    let mut score = 10000.0;

    for m in available_moves(board) {
        if let Some(capture) = board.piece_on(m.get_dest()) {
            score = f32::min(score, value_of_piece(capture));
        } else {
            score = f32::min(score, 0.0);
        }
    }


    if score < 0.0001 {
        eval_generous(board, color)
    } else {
        println!("Insist 2 score: {}", score);
        10000.0 + score
    }
}

pub fn eval_insist_3(board: &Board, color: Color) -> f32 {
    if board.side_to_move() == color {
        panic!("Insist 3 evaluator should only be used for the opponent!");
    }

    let status = board.status();

    if status == BoardStatus::Checkmate {
        return -10e20;
    } else if status == BoardStatus::Stalemate {
        return -10e10;
    }

    let mut score = 0.0;

    let moves = available_moves(board);

    for m in moves.iter() {
        if let Some(capture) = board.piece_on(m.get_dest()) {
            score += 1.0 / moves.len() as f32 * value_of_piece(capture);
        }
    }

    score as f32
}