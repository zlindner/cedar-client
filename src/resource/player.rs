use crate::component::Transform;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub grounded: bool,
    pub parts: Vec<PlayerPart>,
}

impl Player {
    pub fn new(x: f32, y: f32, parts: Vec<PlayerPart>) -> Self {
        Self {
            x,
            y,
            velocity_x: 0.0,
            velocity_y: 0.0,
            grounded: true,
            parts,
        }
    }
}

pub struct PlayerPart {
    pub sprite_index: usize,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl PlayerPart {
    pub fn transform(&self, player: &Player, current_transform: Transform) -> Transform {
        Transform {
            x: player.x + self.offset_x,
            y: player.y + self.offset_y,
            ..current_transform
        }
    }
}
