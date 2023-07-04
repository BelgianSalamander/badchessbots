use chess::{ChessMove, Board, Piece, MoveGen, Rank, File, BoardStatus};

pub fn rank_to_char(rank: Rank) -> char {
    match rank {
        Rank::First => '1',
        Rank::Second => '2',
        Rank::Third => '3',
        Rank::Fourth => '4',
        Rank::Fifth => '5',
        Rank::Sixth => '6',
        Rank::Seventh => '7',
        Rank::Eighth => '8',
    }
}

pub fn file_to_char(file: File) -> char {
    match file {
        File::A => 'a',
        File::B => 'b',
        File::C => 'c',
        File::D => 'd',
        File::E => 'e',
        File::F => 'f',
        File::G => 'g',
        File::H => 'h',
    }
}

pub fn move_to_SAN(board: &Board, m: ChessMove) -> String {
    //First check for castling

    if Some(Piece::King) == board.piece_on(m.get_source()) {
        let start_file = m.get_source().get_file().to_index();
        let end_file = m.get_dest().get_file().to_index();
        
        if (start_file as i32 - end_file as i32).abs() > 1 {
            if start_file < end_file {
                return String::from("O-O");
            } else {
                return String::from("O-O-O");
            }
        }
    }

    let mut san = String::new();

    let piece = board.piece_on(m.get_source()).unwrap();

    match piece {
        Piece::Bishop => san.push('B'),
        Piece::King => san.push('K'),
        Piece::Knight => san.push('N'),
        Piece::Queen => san.push('Q'),
        Piece::Rook => san.push('R'),
        _ => {}
    }

    let all_moves = MoveGen::new_legal(&board);
    let same_dest = all_moves.filter(|x| x.get_dest() == m.get_dest());
    let with_same_piece: Vec<_> = same_dest.filter(|x| board.piece_on(x.get_source()) == Some(piece)).collect();

    let same_rank: Vec<_> = with_same_piece.iter().filter(|x| x.get_source().get_rank() == m.get_source().get_rank()).collect();
    let same_file: Vec<_> = with_same_piece.iter().filter(|x| x.get_source().get_file() == m.get_source().get_file()).collect();

    if with_same_piece.len() > 1 {
        if same_rank.len() > 1 {
            san.push(file_to_char(m.get_source().get_file()));
        } else if same_file.len() > 1 {
            san.push(rank_to_char(m.get_source().get_rank()));
        } else {
            san.push(file_to_char(m.get_source().get_file()));
            san.push(rank_to_char(m.get_source().get_rank()));
        }
    }

    if board.piece_on(m.get_dest()) != None {
        if piece == Piece::Pawn && with_same_piece.len() == 1 {
            san.push(file_to_char(m.get_source().get_file()));
        }

        san.push('x');
    }

    san.push(file_to_char(m.get_dest().get_file()));
    san.push(rank_to_char(m.get_dest().get_rank()));

    if m.get_promotion() != None {
        san.push('=');
        match m.get_promotion().unwrap() {
            Piece::Bishop => san.push('B'),
            Piece::Knight => san.push('N'),
            Piece::Queen => san.push('Q'),
            Piece::Rook => san.push('R'),
            _ => {}
        }
    }

    let resulting_board = board.make_move_new(m);

    if resulting_board.status() == BoardStatus::Checkmate {
        san.push('#');
    } else if resulting_board.checkers().0 != 0 {
        san.push('+');
    }

    san
}