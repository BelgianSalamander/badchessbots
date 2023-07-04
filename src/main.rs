pub mod gui;
pub mod alg;
pub mod util;

use std::sync::{Arc, Mutex};

use alg::chess_alg::RandomChessAlgorithm;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::graphics::{self, Color};
use ggez::event::{self, EventHandler};
use gui::chess_display::{ChessDisplay, PlayerType};
use ggez::conf::{WindowSetup, WindowMode};
use gui::main_gui::MainGUI;

fn main() {
    let mut cb = ContextBuilder::new("chess_arena", "Salamander")
        .window_setup(WindowSetup::default().title("Chess Arena"))
        .window_mode(WindowMode::default().dimensions(800.0, 600.0).resizable(true));

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = std::path::PathBuf::from(manifest_dir);
        path.push("res");

        println!("Resource path: {:?}", path);

        cb = cb.add_resource_path(path);
    } else {
        cb = cb.add_resource_path(std::path::PathBuf::from("./res"));
    }

    let (mut ctx, event_loop) = cb.build()
        .expect("aieee, could not create ggez context!");

    let gui = MainGUI::new(&mut ctx);

    event::run(ctx, event_loop, gui);
}