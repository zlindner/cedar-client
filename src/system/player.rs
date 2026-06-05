use crate::{
    component::Camera,
    resource::{GameKey, MapPhysics, Player},
    state::State,
};

// HeavenClient advances movement in 8ms physics ticks and stores velocity in
// pixels per physics tick. Keep these constants in that unit until we have a
// fuller physics object/foothold port.
const DEFAULT_SPEED_STAT: f32 = 100.0;
const DEFAULT_JUMP_STAT: f32 = 100.0;
const WALK_FORCE: f32 = 0.05 + 0.11 * DEFAULT_SPEED_STAT / 100.0;
const JUMP_FORCE: f32 = 1.0 + 3.5 * DEFAULT_JUMP_STAT / 100.0;
const GRAVITY_FORCE: f32 = 0.14;
const FRICTION: f32 = 0.3;
const GROUND_SLIP: f32 = 3.0;
const AIR_TURN_BRAKE: f32 = 0.025;
const CAMERA_VERTICAL_TARGET_OFFSET: f32 = -32.0;
const CAMERA_BOTTOM_PADDING: f32 = 120.0;
const CAMERA_FOLLOW_MIN_DELTA: f32 = 5.0;
const CAMERA_FOLLOW_SPEED: f32 = 18.0;

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

        let previous_y = player.y;
        let previous_grounded = player.grounded;
        move_normal(&mut player, walk_force, jump_force);
        apply_map_ground(
            &mut player,
            previous_y,
            previous_grounded,
            state.get_resource::<MapPhysics>().as_deref(),
        );

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
    let (player_x, player_y) = (player.x, player.y);
    drop(player);

    let Some(map_physics) = state.get_resource::<MapPhysics>() else {
        return;
    };
    let walls = map_physics.walls();
    let mut borders = map_physics.borders();
    borders.max -= CAMERA_BOTTOM_PADDING;
    drop(map_physics);

    let Some(mut camera) = state.get_resource_mut::<Camera>() else {
        return;
    };

    let camera_x = follow_camera_axis(camera.center_x(), player_x, camera.width());
    let camera_y = follow_camera_axis(
        camera.center_y(),
        player_y + CAMERA_VERTICAL_TARGET_OFFSET,
        camera.height(),
    );

    // Match HeavenClient's camera behavior: ease toward the player every tick
    // once the target is at least a few pixels away, then clamp to map bounds.
    camera.center_on_clamped(camera_x, camera_y, walls, borders);
}

fn follow_camera_axis(current: f32, target: f32, viewport_size: f32) -> f32 {
    let delta = target - current;
    if delta.abs() < CAMERA_FOLLOW_MIN_DELTA {
        current
    } else {
        current + delta * (CAMERA_FOLLOW_SPEED / viewport_size)
    }
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
            let slope = player.ground_slope.clamp(-0.5, 0.5);
            hacc -= FRICTION * inertia + 0.1 * (1.0 + slope * -inertia) * inertia;
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

fn apply_map_ground(
    player: &mut Player,
    previous_y: f32,
    previous_grounded: bool,
    map_physics: Option<&MapPhysics>,
) {
    let Some(map_physics) = map_physics else {
        return;
    };

    player.x = map_physics.clamp_x(player.x);

    if previous_grounded && player.velocity_y >= 0.0 {
        if let Some(ground) =
            map_physics.ground_for_player(player.foothold_id, player.x, previous_y, true)
        {
            snap_to_ground(player, ground);
            return;
        }

        player.grounded = false;
        return;
    }

    let Some(ground) = map_physics.ground_below(player.x, previous_y.min(player.y)) else {
        player.grounded = false;
        return;
    };

    if player.velocity_y >= 0.0 && previous_y <= ground.y && player.y >= ground.y {
        snap_to_ground(player, ground);
    } else {
        player.grounded = false;
    }
}

fn snap_to_ground(player: &mut Player, ground: crate::resource::Ground) {
    player.y = ground.y;
    player.velocity_y = 0.0;
    player.grounded = true;
    player.foothold_id = ground.id;
    player.foothold_layer = ground.layer;
    player.ground_slope = ground.slope;
}
