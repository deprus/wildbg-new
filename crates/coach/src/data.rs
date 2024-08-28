use engine::inputs::InputsGen;
use engine::position::Position;
use engine::probabilities::Probabilities;
use serde::{Deserialize, Serialize};

/// Position ID and 5 probabilities meant to be serialized to CSV to keep training data for a longer time.
///
/// We don't use the 6 probabilities format to be more compatible with other backgammon programs.
/// `win` includes the chance to win gammon or backgammon.
/// `win_g` and `lose_g` include the chance to win or lose backgammon.
#[derive(Debug, Deserialize, Serialize)]
pub struct PositionRecord {
    position_id: String,
    win: f32,
    win_g: f32,
    lose_g: f32,
}

impl PositionRecord {
    pub fn new(position: &Position, probabilities: &Probabilities) -> Self {
        PositionRecord {
            position_id: position.position_id(),
            win: probabilities.win_normal + probabilities.win_gammon,
            win_g: probabilities.win_gammon,
            lose_g: probabilities.lose_gammon,
        }
    }

    pub fn csv_header() -> Vec<String> {
        vec![
            "position_id".to_owned(),
            "win".to_owned(),
            "win_g".to_owned(),
            "lose_g".to_owned(),
        ]
    }
}

#[derive(Debug, Serialize)]
pub struct InputsRecord {
    win_normal: f32,
    win_gammon: f32,
    lose_normal: f32,
    lose_gammon: f32,
    inputs: Vec<f32>,
}

impl InputsRecord {
    pub fn new<T: InputsGen>(record: &PositionRecord, inputs_gen: &T) -> Self {
        let position = Position::from_id(record.position_id.clone());
        InputsRecord {
            win_normal: record.win - record.win_g,
            win_gammon: record.win_g,
            lose_normal: 1.0 - record.win - record.lose_g,
            lose_gammon: record.lose_g,
            inputs: inputs_gen.inputs_for_single(&position),
        }
    }
}
