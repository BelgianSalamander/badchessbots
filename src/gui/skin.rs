use chess::{Piece, Color};
use ggez::{graphics, Context};

#[derive(Debug)]
pub struct PieceSkin {
    pub name: String,

    pub white_king: graphics::Image,
    pub white_queen: graphics::Image,
    pub white_rook: graphics::Image,
    pub white_bishop: graphics::Image,
    pub white_knight: graphics::Image,
    pub white_pawn: graphics::Image,

    pub black_king: graphics::Image,
    pub black_queen: graphics::Image,
    pub black_rook: graphics::Image,
    pub black_bishop: graphics::Image,
    pub black_knight: graphics::Image,
    pub black_pawn: graphics::Image,
}

impl PieceSkin {
    pub fn load(ctx: &mut Context, name: &str) -> Self {
        let white_king = graphics::Image::from_path(ctx, format!("/chess-skins/{}/white-king.png", name)).unwrap();
        let white_queen = graphics::Image::from_path(ctx, format!("/chess-skins/{}/white-queen.png", name)).unwrap();
        let white_rook = graphics::Image::from_path(ctx, format!("/chess-skins/{}/white-rook.png", name)).unwrap();
        let white_bishop = graphics::Image::from_path(ctx, format!("/chess-skins/{}/white-bishop.png", name)).unwrap();
        let white_knight = graphics::Image::from_path(ctx, format!("/chess-skins/{}/white-knight.png", name)).unwrap();
        let white_pawn = graphics::Image::from_path(ctx, format!("/chess-skins/{}/white-pawn.png", name)).unwrap();

        let black_king = graphics::Image::from_path(ctx, format!("/chess-skins/{}/black-king.png", name)).unwrap();
        let black_queen = graphics::Image::from_path(ctx, format!("/chess-skins/{}/black-queen.png", name)).unwrap();
        let black_rook = graphics::Image::from_path(ctx, format!("/chess-skins/{}/black-rook.png", name)).unwrap();
        let black_bishop = graphics::Image::from_path(ctx, format!("/chess-skins/{}/black-bishop.png", name)).unwrap();
        let black_knight = graphics::Image::from_path(ctx, format!("/chess-skins/{}/black-knight.png", name)).unwrap();
        let black_pawn = graphics::Image::from_path(ctx, format!("/chess-skins/{}/black-pawn.png", name)).unwrap();

        PieceSkin {
            name: name.to_string(),

            white_king,
            white_queen,
            white_rook,
            white_bishop,
            white_knight,
            white_pawn,

            black_king,
            black_queen,
            black_rook,
            black_bishop,
            black_knight,
            black_pawn,
        }
    }

    pub fn get_piece_image<'a>(&'a self, piece: Piece, color: Color) -> &'a graphics::Image {
        match (piece, color) {
            (Piece::King, Color::White) => &self.white_king,
            (Piece::Queen, Color::White) => &self.white_queen,
            (Piece::Rook, Color::White) => &self.white_rook,
            (Piece::Bishop, Color::White) => &self.white_bishop,
            (Piece::Knight, Color::White) => &self.white_knight,
            (Piece::Pawn, Color::White) => &self.white_pawn,

            (Piece::King, Color::Black) => &self.black_king,
            (Piece::Queen, Color::Black) => &self.black_queen,
            (Piece::Rook, Color::Black) => &self.black_rook,
            (Piece::Bishop, Color::Black) => &self.black_bishop,
            (Piece::Knight, Color::Black) => &self.black_knight,
            (Piece::Pawn, Color::Black) => &self.black_pawn,
        }
    }
}