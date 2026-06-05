use crate::{
    component::Camera,
    resource::{GameKey, Player},
    state::State,
};

// HeavenClient advances movement in 8ms physics ticks and stores velocity in
// pixels per physics tick. Keep these constants in that unit until we have a
// fuller physics object/foothold port.
const FLOOR_Y: f32 = 350.0;
const DEFAULT_SPEED_STAT: f32 = 100.0;
const DEFAULT_JUMP_STAT: f32 = 100.0;
const WALK_FORCE: f32 = 0.05 + 0.11 * DEFAULT_SPEED_STAT / 100.0;
const JUMP_FORCE: f32 = 1.0 + 3.5 * DEFAULT_JUMP_STAT / 100.0;
const GRAVITY_FORCE: f32 = 0.14;
const FRICTION: f32 = 0.3;
const GROUND_SLIP: f32 = 3.0;
const AIR_TURN_BRAKE: f32 = 0.025;

pub fn player_movement_system(state: &mut State) {
    let (left, right, jump) = {
        let mut keyboard = state.keyboard();
        (
            keyboard.key_down(GameKey::Left),
            keyboard.key_down(GameKey::Right),
            keyboard.consume_key_pressed(GameKey::Jump),
        )
    };

    let sprite_updates = {
        let Some(mut player) = state.get_resource_mut::<Player>() else {
            return;
        };

        let jump_force = if jump && player.grounded {
            -JUMP_FORCE
        } else {
            0.0
        };
        let walk_force = if player.grounded {
            match (left, right) {
                (true, false) => -WALK_FORCE,
                (false, true) => WALK_FORCE,
                _ => 0.0,
            }
        } else {
            apply_air_turn_brake(&mut player, left, right);
            0.0
        };

        move_normal(&mut player, walk_force, jump_force);
        limit_to_test_floor(&mut player);

        player
            .parts
            .iter()
            .map(|part| {
                let current_transform = state.sprites[part.sprite_index].transform();
                (
                    part.sprite_index,
                    part.transform(&player, current_transform),
                )
            })
            .collect::<Vec<_>>()
    };

    for (sprite_index, transform) in sprite_updates {
        state.sprites[sprite_index].set_transform(transform);
    }
}

pub fn camera_follow_system(state: &mut State) {
    let Some(player) = state.get_resource::<Player>() else {
        return;
    };
    let player_x = player.x;
    drop(player);

    let Some(mut camera) = state.get_resource_mut::<Camera>() else {
        return;
    };

    // Keep the camera vertically stable for now. Maple-style camera follow should
    // eventually use map bounds/dead zones rather than tracking jump height.
    camera.center_on(player_x, FLOOR_Y);
}

fn move_normal(player: &mut Player, hforce: f32, vforce: f32) {
    let mut hacc = 0.0;
    let mut vacc = 0.0;

    if player.grounded {
        vacc += vforce;
        hacc += hforce;

        if hacc == 0.0 && player.velocity_x < 0.1 && player.velocity_x > -0.1 {
            player.velocity_x = 0.0;
        } else {
            let inertia = player.velocity_x / GROUND_SLIP;
            hacc -= FRICTION * inertia;
        }
    } else {
        vacc += GRAVITY_FORCE;
    }

    player.velocity_x += hacc;
    player.velocity_y += vacc;
    player.x += player.velocity_x;
    player.y += player.velocity_y;
}

fn apply_air_turn_brake(player: &mut Player, left: bool, right: bool) {
    if left && player.velocity_x > 0.0 {
        player.velocity_x = (player.velocity_x - AIR_TURN_BRAKE).max(0.0);
    } else if right && player.velocity_x < 0.0 {
        player.velocity_x = (player.velocity_x + AIR_TURN_BRAKE).min(0.0);
    }
}

fn limit_to_test_floor(player: &mut Player) {
    if player.y >= FLOOR_Y {
        player.y = FLOOR_Y;
        player.velocity_y = 0.0;
        player.grounded = true;
    } else {
        player.grounded = false;
    }
}
