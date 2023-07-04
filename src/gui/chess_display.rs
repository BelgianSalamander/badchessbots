use std::sync::{Arc, Mutex};
use std::thread;

use ggez::event::{EventHandler, MouseButton};
use ggez::graphics::{Canvas, Color, Text, Rect, Mesh, TextFragment, TextAlign, TextLayout};
use ggez::{event, graphics, Context, GameError, GameResult};

use chess::{Board, BoardStatus, ChessMove, File, MoveGen, Piece, Rank, Square, ALL_SQUARES};

use crate::alg::chess_alg::ChessAlgorithm;
use crate::util::move_to_SAN;

use super::skin::PieceSkin;

const BACKGROUND_COLOR: Color = Color::new(0.3, 0.3, 0.3, 1.0);

const BOARD_WHITE: Color = Color::new(227.0 / 255.0, 220.0 / 255.0, 138.0 / 255.0, 1.0);
const BOARD_BLACK: Color = Color::new(128.0 / 255.0, 69.0 / 255.0, 33.0 / 255.0, 1.0);

const BOARD_SELECTED_WHITE: Color = Color::new(188.0 / 255.0, 222.0 / 255.0, 115.0 / 255.0, 1.0);
const BOARD_SELECTED_BLACK: Color = Color::new(61.0 / 255.0, 92.0 / 255.0, 21.0 / 255.0, 1.0);

#[derive(Debug)]
pub enum GameOutcome {
    Checkmate(chess::Color),
    Stalemate,
    InsufficientMaterial,
    DrawByRepetition,
    DrawBy50MoveRule,
}

impl GameOutcome {
    pub fn get_text(&self) -> &'static str {
        match self {
            GameOutcome::Checkmate(color) => match color {
                chess::Color::White => "White wins by checkmate",
                chess::Color::Black => "Black wins by checkmate",
            },
            GameOutcome::Stalemate => "Stalemate",
            GameOutcome::InsufficientMaterial => "Draw by insufficient material",
            GameOutcome::DrawByRepetition => "Draw by repetition",
            GameOutcome::DrawBy50MoveRule => "Draw by 50 move rule",
        }
    }
}

#[derive(Debug)]
struct BoardDimensions {
    x_offset: f32,
    y_offset: f32,

    square_size: f32,
}

#[derive(Debug)]
pub enum PlayerType {
    Human,
    Computer(Arc<Mutex<dyn ChessAlgorithm>>),
}

impl PlayerType {
    pub fn is_human(&self) -> bool {
        match self {
            PlayerType::Human => true,
            _ => false,
        }
    }

    pub fn is_computer(&self) -> bool {
        match self {
            PlayerType::Computer(_) => true,
            _ => false,
        }
    }

    pub fn computer<T: ChessAlgorithm + 'static>(algorithm: T) -> PlayerType {
        PlayerType::Computer(Arc::new(Mutex::new(algorithm)))
    }
}

fn is_insufficient_material(board: &Board) -> bool {
    let mut res = true;

    for color in vec![chess::Color::White, chess::Color::Black] {
        let mut num_pawns = 0;
        let mut num_queens = 0;
        let mut num_rooks = 0;
        let mut num_bishops = 0;
        let mut num_knights = 0;

        for square in ALL_SQUARES {
            if let Some(piece) = board.piece_on(square) {
                let piece_color = board.color_on(square).unwrap();
                if piece_color == color {
                    match piece {
                        chess::Piece::Pawn => num_pawns += 1,
                        chess::Piece::Queen => num_queens += 1,
                        chess::Piece::Rook => num_rooks += 1,
                        chess::Piece::Bishop => num_bishops += 1,
                        chess::Piece::Knight => num_knights += 1,
                        _ => (),
                    }
                }
            }
        }

        if num_pawns > 0
            || num_queens > 0
            || num_rooks > 0
            || num_bishops > 1
            || num_knights > 2
            || (num_knights > 1 && num_bishops > 1)
        {
            res = false;
            break;
        }
    }

    res
}

#[derive(Debug)]
pub struct ChessDisplay {
    pub board: Board,
    board_dimensions: BoardDimensions,

    white_player: PlayerType,
    black_player: PlayerType,

    skin: PieceSkin,

    selected_square: Option<(u8, u8)>,

    next_move_future: Arc<Mutex<Option<ChessMove>>>,

    history: Vec<Board>,
    reversable_moves: u32,

    outcome: Option<GameOutcome>,
}

impl ChessDisplay {
    pub fn new(
        ctx: &mut Context,
        white_player: PlayerType,
        black_player: PlayerType,
    ) -> ChessDisplay {
        let mut res = ChessDisplay {
            board: Board::default(),
            board_dimensions: BoardDimensions {
                x_offset: 0.0,
                y_offset: 0.0,
                square_size: 50.0,
            },

            white_player,
            black_player,

            skin: PieceSkin::load(ctx, "default"),

            selected_square: None,

            next_move_future: Arc::new(Mutex::new(None)),

            history: Vec::new(),
            reversable_moves: 0,

            outcome: None,
        };

        res.on_new_move();

        res
    }

    fn update_dims(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let board_size = width.min(height);

        self.board_dimensions.square_size = board_size / 8.0;
        self.board_dimensions.x_offset = x + (width - board_size) / 2.0;
        self.board_dimensions.y_offset = y + (height - board_size) / 2.0;
    }

    fn chess_to_screen(&self, rank: u8, file: u8) -> (f32, f32) {
        let x = self.board_dimensions.x_offset + self.board_dimensions.square_size * file as f32;
        let y =
            self.board_dimensions.y_offset + self.board_dimensions.square_size * (7 - rank) as f32;

        (x, y)
    }

    fn screen_to_chess(&self, x: f32, y: f32) -> Option<(u8, u8)> {
        let file =
            ((x - self.board_dimensions.x_offset) / self.board_dimensions.square_size) as i32;
        let rank =
            ((y - self.board_dimensions.y_offset) / self.board_dimensions.square_size) as i32;

        if file < 0 || file > 7 || rank < 0 || rank > 7 {
            None
        } else {
            Some((7 - rank as u8, file as u8))
        }
    }

    fn get_square_color(&self, rank: u8, file: u8) -> Color {
        let even = (rank + file) % 2 == 0;

        if self.selected_square == Some((rank, file)) {
            if even {
                BOARD_SELECTED_BLACK
            } else {
                BOARD_SELECTED_WHITE
            }
        } else if even {
            BOARD_BLACK
        } else {
            BOARD_WHITE
        }
    }

    fn draw_blank_board(&self, ctx: &mut Context, canvas: &mut Canvas) {
        let rectangle = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0, 0.0, 1.0, 1.0),
            Color::WHITE,
        )
        .unwrap();

        for x in 0..8 {
            for y in 0..8 {
                let color = self.get_square_color(y, x);

                let (draw_x, draw_y) = self.chess_to_screen(y, x);

                canvas.draw(
                    &rectangle,
                    graphics::DrawParam::default()
                        .dest([draw_x, draw_y])
                        .scale([
                            self.board_dimensions.square_size,
                            self.board_dimensions.square_size,
                        ])
                        .color(color),
                );
            }
        }
    }

    fn draw_pieces(&self, ctx: &mut Context, canvas: &mut Canvas) {
        for x in 0..8 {
            let file = File::from_index(x);

            for y in 0..8 {
                let rank = Rank::from_index(7 - y);

                let square = Square::make_square(rank, file);

                if let Some(piece) = self.board.piece_on(square) {
                    let color = self.board.color_on(square).unwrap();

                    let piece_image = self.skin.get_piece_image(piece, color);

                    let x = self.board_dimensions.x_offset
                        + self.board_dimensions.square_size * x as f32;
                    let y = self.board_dimensions.y_offset
                        + self.board_dimensions.square_size * y as f32;

                    canvas.draw(
                        piece_image,
                        graphics::DrawParam::default().dest([x, y]).scale([
                            self.board_dimensions.square_size / piece_image.width() as f32,
                            self.board_dimensions.square_size / piece_image.height() as f32,
                        ]),
                    );
                }
            }
        }
    }

    fn generate_moves(&self) -> Vec<(ChessMove, (u8, u8))> {
        //If the current player is a computer, there is nothihng that should be returned
        if self.current_player().is_computer() {
            return vec![];
        }

        if let Some((rank, file)) = self.selected_square {
            let square = Square::make_square(
                Rank::from_index(rank as usize),
                File::from_index(file as usize),
            );

            MoveGen::new_legal(&self.board)
                .filter_map(|m| {
                    if m.get_source() == square {
                        Some((
                            m,
                            (
                                m.get_dest().get_rank().to_index() as u8,
                                m.get_dest().get_file().to_index() as u8,
                            ),
                        ))
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn draw_available_moves(&self, ctx: &mut Context, canvas: &mut Canvas) {
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            [0.0, 0.0],
            self.board_dimensions.square_size * 0.25,
            0.25,
            Color::from_rgba(255, 0, 0, 200),
        )
        .unwrap();

        for (_, (rank, file)) in self.generate_moves() {
            let (x, y) = self.chess_to_screen(rank, file);

            canvas.draw(
                &circle,
                graphics::DrawParam::default()
                    .dest([
                        x + self.board_dimensions.square_size / 2.0,
                        y + self.board_dimensions.square_size / 2.0,
                    ])
                    .scale([1.0, 1.0]),
            );
        }
    }

    fn current_player<'a>(&'a self) -> &'a PlayerType {
        if self.board.side_to_move() == chess::Color::White {
            &self.white_player
        } else {
            &self.black_player
        }
    }

    fn detect_draw(&mut self) -> bool {
        if self.board.status() == BoardStatus::Stalemate {
            self.outcome = Some(GameOutcome::Stalemate);
            return true;
        }

        if is_insufficient_material(&self.board) {
            self.outcome = Some(GameOutcome::InsufficientMaterial);
            return true;
        }

        let mut num_occurences = 0;

        for pos in self.history.iter() {
            if *pos == self.board {
                num_occurences += 1;
            }
        }

        if num_occurences >= 3 {
            self.outcome = Some(GameOutcome::DrawByRepetition);
            return true;
        }

        if self.reversable_moves >= 50 {
            self.outcome = Some(GameOutcome::DrawBy50MoveRule);
            return true;
        }

        false
    }

    fn on_new_move(&mut self) {
        if self.board.status() == BoardStatus::Checkmate {
            println!("Checkmate!");
            match self.board.side_to_move() {
                chess::Color::White => {
                    println!("Black wins!");
                    self.outcome = Some(GameOutcome::Checkmate(chess::Color::Black));
                }
                chess::Color::Black => {
                    println!("White wins!");
                    self.outcome = Some(GameOutcome::Checkmate(chess::Color::White));
                }
            }

            return;
        }

        if self.detect_draw() {
            println!("Draw!");

            return;
        }

        self.try_launch_engine();
    }

    fn try_launch_engine(&mut self) {
        if self.outcome.is_some() {
            return;
        }

        if self.current_player().is_computer() {
            let board = self.board.clone();

            if let PlayerType::Computer(engine) = self.current_player() {
                let engine = engine.clone();

                {
                    self.next_move_future.lock().unwrap().take();
                }

                let output = self.next_move_future.clone();

                thread::spawn(move || {
                    let mut engine = engine.lock().unwrap();

                    let m = engine.get_move(board);

                    output.lock().unwrap().replace(m);
                });
            }
        }
    }

    fn do_move(&mut self, m: ChessMove) {
        println!("Move: {}", move_to_SAN(&self.board, m));

        let mut reversable = true;

        if Some(Piece::Pawn) == self.board.piece_on(m.get_source()) {
            reversable = false;
        } else if self.board.piece_on(m.get_dest()).is_some() {
            reversable = false;
        }

        if reversable {
            self.reversable_moves += 1;
        } else {
            self.reversable_moves = 0;
        }

        self.history.push(self.board.clone());
        self.board = self.board.make_move_new(m);

        self.on_new_move();
    }

    pub fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let m = { self.next_move_future.lock().unwrap().take() };

        if let Some(m) = m {
            self.do_move(m);
        }

        Ok(())
    }

    pub fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas, x: f32, y: f32, w: f32, h: f32) -> Result<(), GameError> {
        self.update_dims(x, y, w , h);

        self.draw_blank_board(ctx, canvas);
        self.draw_pieces(ctx, canvas);
        self.draw_available_moves(ctx, canvas);

        if let Some(outcome) = &self.outcome {
            let mut text = Text::default();

            text.set_bounds([self.board_dimensions.square_size * 7.8, 10000000.0]);
            
            text.add(TextFragment::new(outcome.get_text()).scale(60.0).color(Color::BLACK));
            text.add(TextFragment::new("\nPress ESC to return to main menu").scale(25.0).color(Color::new(0.4, 0.4, 0.4, 1.0)));
            
            text.set_layout(TextLayout::center());

            let dims = text.measure(ctx)?;

            let background_bounds = Rect::new(
                self.board_dimensions.x_offset + self.board_dimensions.square_size * 4.0 - dims.x / 2.0 - 10.0,
                self.board_dimensions.y_offset + self.board_dimensions.square_size * 4.0 - dims.y / 2.0 - 10.0,
                dims.x + 20.0,
                dims.y + 20.0,
            );

            let background = Mesh::new_rounded_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                background_bounds,
                5.0,
                [1.0, 1.0, 1.0, 0.5].into(),
            )?;

            let border = Mesh::new_rounded_rectangle(
                ctx,
                graphics::DrawMode::stroke(5.0),
                background_bounds,
                5.0,
                [0.0, 0.0, 0.0, 1.0].into(),
            )?;

            canvas.draw(&background, graphics::DrawParam::default());
            canvas.draw(&border, graphics::DrawParam::default());

            let x = self.board_dimensions.x_offset + self.board_dimensions.square_size * 4.0;
            let y = self.board_dimensions.y_offset + self.board_dimensions.square_size * 4.0;

            canvas.draw(
                &text,
                graphics::DrawParam::default()
                    .dest([x, y])
                    .color([1.0, 1.0, 1.0, 1.0])
                    .scale([1.0, 1.0]),
            );
        }

        Ok(())
    }

    pub fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), GameError> {
        if button == MouseButton::Left {
            if let Some(game_pos) = self.screen_to_chess(x, y) {
                for (m, (rank, file)) in self.generate_moves() {
                    if (rank, file) == game_pos {
                        self.do_move(m);
                        self.selected_square = None;
                        return Ok(());
                    }
                }

                if let Some((rank, file)) = self.selected_square {
                    if (rank, file) == game_pos {
                        self.selected_square = None;
                    } else {
                        self.selected_square = Some(game_pos);
                    }
                } else {
                    self.selected_square = Some(game_pos);
                }
            }
        }

        Ok(())
    }
}

