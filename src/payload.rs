use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HugCommand {
    JoinRoom { key: String },
    JoinRandom,
    CreateRoom,
    Leave,
    Push { payload: Payload },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HugEvent {
    Joined { is_primary: bool },
    RoomCreated { key: String },
    NotFound,
    Push { payload: Payload },
}

#[derive(Serialize, Deserialize)]
pub enum Payload {
    HandControl {
        left: Vec2,
        right: Vec2,
    },
    Sync {
        player1_head: Vec3,
        player2_head: Vec3,
        player1_hip: Vec3,
        player2_hip: Vec3,
        player1_hand_left: Vec3,
        player2_hand_left: Vec3,
        player1_hand_right: Vec3,
        player2_hand_right: Vec3,
    },
    Name(String),
}

type Vec2 = [f32; 2];
type Vec3 = [f32; 3];
