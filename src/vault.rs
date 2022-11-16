use std::sync::Mutex;

// use clightningrpc::LightningRPC;
use crate::{cashu, BOARD_SIZE};
use rand::Rng;
use rocket::{get, http::Status, serde::json::Json, State};
use serde::Serialize;

#[derive(Debug)]
pub struct BattleConfig {
    pub board: Vec<Vec<(u8, u32)>>,
}

#[derive(Serialize, Debug)]
pub struct LandResponse {
    success: bool,
    x: u8,
    y: u8,
    message: String,
}

fn very_tustworthy_rng() -> (u8, u8) {
    let mut rng = rand::thread_rng();
    (
        rng.gen_range(0..BOARD_SIZE) as u8,
        rng.gen_range(0..BOARD_SIZE) as u8,
    )
}

#[get("/land?<token>&<color>")]
pub fn land(
    state: &State<Mutex<BattleConfig>>,
    token: String,
    color: u8,
) -> (Status, Json<LandResponse>) {
    // Hints for submitting an ln invoice
    if &token[0..2] == "ln" {
        return (
            Status::InternalServerError,
            Json(LandResponse {
                success: false,
                x: 0,
                y: 0,
                message: "🚬 Close but no cigar.".to_string(),
            }),
        );
    }
    // Must be a valid color
    if (color > 4) | (color == 0) {
        return (
            Status::InternalServerError,
            Json(LandResponse {
                success: false,
                x: 0,
                y: 0,
                message: "Invalid color selected?!".to_string(),
            }),
        );
    }

    let amount_paid = cashu::cashu_receive(&token);
    return if amount_paid == 0 {
        (
            Status::InternalServerError,
            Json(LandResponse {
                success: false,
                x: 0,
                y: 0,
                message: "⚠️ Key Rejected ⚠️".to_string(),
            }),
        )
    } else {
        let (x, y) = very_tustworthy_rng();
        let mut lock = state.lock().unwrap();
        let (square_color, square_amount) = lock.board[y as usize][x as usize];
        if color == square_color {
            let token = cashu::cashu_send(square_amount);
            lock.board[y as usize][x as usize] = (0, 0);
            return (
                Status::Accepted,
                Json(LandResponse {
                    success: true,
                    x,
                    y,
                    message: format!("🎈 Valid Key ({}, {}). 🎈\nUnlocked:\n\n{}", x, y, token)
                        .to_string(),
                }),
            );
        } else {
            lock.board[y as usize][x as usize] = (color, amount_paid + square_amount);
            if square_color != 0 {
                return (
                    Status::Accepted,
                    Json(LandResponse {
                        success: true,
                        x,
                        y,
                        message: format!("💔 Key Failed. Valid key was ({}, {})", x, y).to_string(),
                    }),
                );
            } else {
                return (
                    Status::Accepted,
                    Json(LandResponse {
                        success: true,
                        x,
                        y,
                        message: format!("🔒 Key not ready. 🔒 ({}, {})", x, y).to_string(),
                    }),
                );
            }
        }
    };
}

#[derive(Serialize, Debug)]
pub struct BoardResponse {
    colors: Vec<Vec<u8>>,
    amounts: Vec<Vec<u32>>,
}

#[get("/board")]
pub fn board(state: &State<Mutex<BattleConfig>>) -> (Status, Json<BoardResponse>) {
    let lock = state.lock().unwrap();
    let mut colors = vec![];
    let mut amounts = vec![];
    for row in lock.board.iter() {
        colors.push(row.iter().map(|sq| sq.0).collect());
        amounts.push(row.iter().map(|sq| sq.1).collect());
    }

    (Status::Accepted, Json(BoardResponse { colors, amounts }))
}
