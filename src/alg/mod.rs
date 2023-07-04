use crate::gui::chess_display::PlayerType;

use self::{chess_alg::{RandomChessAlgorithm, FirstMoveAlgorithm, AlphabeticalChessAlgorithm}, one_lookahead::SingleLookaheadEngine, evaluators::{eval_matching_colors, eval_opposite_colors, eval_pacifist}};

pub mod chess_alg;
pub mod one_lookahead;
pub mod evaluators;
pub mod tree_search;

pub type PlayerTypeSupplier = fn(chess::Color) -> PlayerType;

pub const ALL_PLAYER_TYPES: [(&str, PlayerTypeSupplier); 12] = [
    ("Human", |_| {PlayerType::Human}),
    ("Random", |_| {PlayerType::computer(RandomChessAlgorithm)}),
    ("Matching", |color| {PlayerType::computer(SingleLookaheadEngine::new(color, eval_matching_colors))}),
    ("Opposite", |color| {PlayerType::computer(SingleLookaheadEngine::new(color, eval_opposite_colors))}),
    ("Pacifist", |color| {PlayerType::computer(SingleLookaheadEngine::new(color, eval_pacifist))}),
    ("First", |_| PlayerType::computer(FirstMoveAlgorithm)),
    ("Alphabetical", |_| PlayerType::computer(AlphabeticalChessAlgorithm)),
    ("Huddle", |color| PlayerType::computer(SingleLookaheadEngine::new(color, evaluators::eval_huddle))),
    ("Swarm", |color| PlayerType::computer(SingleLookaheadEngine::new(color, evaluators::eval_swarm))),
    ("Generous", |color| PlayerType::computer(SingleLookaheadEngine::new(color, evaluators::eval_generous))),
    ("I Insist 2", |color| PlayerType::computer(SingleLookaheadEngine::new(color, evaluators::eval_insist_2))),
    ("I Insist 3", |color| PlayerType::computer(SingleLookaheadEngine::new(color, evaluators::eval_insist_3))),
];