use crate::component::Transform;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub grounded: bool,
    pub foothold_id: u16,
    pub foothold_layer: u8,
    pub ground_slope: f32,
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
            foothold_id: 0,
            foothold_layer: 0,
            ground_slope: 0.0,
            parts,
        }
    }

    pub fn with_ground(mut self, foothold_id: u16, foothold_layer: u8, ground_slope: f32) -> Self {
        self.foothold_id = foothold_id;
        self.foothold_layer = foothold_layer;
        self.ground_slope = ground_slope;
        self
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
