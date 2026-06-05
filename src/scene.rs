use crate::{
    component::{Camera, Colour, Transform},
    graphics::{
        ui::{load_button_images, Button, ButtonState, TextInput},
        Sprite,
    },
    resource::{AssetManager, FontDescriptor, Ground, MapLine, MapPhysics, Player, PlayerPart},
    state::State,
};
use nx_pkg4::{Node, NxNode};

pub trait Scene {
    fn init(&mut self, _state: &mut State) {}
}

#[derive(Default)]
pub struct LoginScene;

impl Scene for LoginScene {
    fn init(&mut self, state: &mut State) {
        init_sprites(state);
        init_buttons(state);
        init_text_inputs(state);
    }
}

#[derive(Default)]
pub struct GameScene;

impl Scene for GameScene {
    fn init(&mut self, state: &mut State) {
        init_test_map(state);
        init_test_character(state);
        init_game_status_bar(state);
    }
}

const TEST_MAP_ID: i32 = 100000000;
const TEST_MAP_VIEW_X: f32 = 0.0;
const TEST_MAP_VIEW_Y: f32 = 0.0;
const TEST_CHARACTER_X: f32 = 4000.0;
const TEST_CHARACTER_SPAWN_Y: f32 = 350.0;
const TEST_CAMERA_INITIAL_Y_OFFSET: f32 = -32.0;
const TEST_CAMERA_BOTTOM_PADDING: f32 = 120.0;
const GAME_STATUS_BAR_Y: f32 = 480.0;
const DEBUG_FOOTHOLDS: bool = true;
const DEBUG_FOOTHOLD_Z: f32 = 16.0;

struct MapSpritePlacement {
    path: String,
    x: f32,
    y: f32,
    z: f32,
    apply_view_offset: bool,
}

struct CharacterPartPlacement {
    path: String,
    x: f32,
    y: f32,
    z: f32,
}

struct CharacterStart {
    x: f32,
    y: f32,
    ground: Option<Ground>,
}

// TODO: we might eventually want sprites to be more complex (animations, hiding, etc.), so we may
// want to create a simple "UiImage" struct or something for these.
fn init_sprites(state: &mut State) {
    let sprites = {
        let mut assets = state
            .get_resource_mut::<AssetManager>()
            .expect("AssetManager should exist");

        vec![
            Sprite::new(
                assets
                    .load_image("Map001.nx/Back/login.img/back/11")
                    .unwrap(),
            )
            .with_transform(Transform::from_xyz(400.0, 300.0, 1.0)),
            Sprite::new(
                assets
                    .load_image("Map001.nx/Back/login.img/back/35")
                    .unwrap(),
            )
            .with_transform(Transform::from_xyz(399.0, 260.0, 2.0)),
            Sprite::new(
                assets
                    .load_image("MapPretty.nx/Back/login.img/ani/16/0")
                    .unwrap(),
            )
            .with_transform(Transform::from_xyz(394.0, 173.0, 2.0)),
            Sprite::new(
                assets
                    .load_image("UI.nx/Login.img/Title/signboard")
                    .unwrap(),
            )
            .with_transform(Transform::from_xyz(391.0, 330.0, 10.0)),
            Sprite::new(assets.load_image("UI.nx/Login.img/Common/frame").unwrap())
                .with_transform(Transform::from_xyz(400.0, 300.0, 10.0)),
        ]
    };

    state.sprites.extend(sprites);
}

fn init_buttons(state: &mut State) {
    let buttons = {
        let mut assets = state
            .get_resource_mut::<AssetManager>()
            .expect("AssetManager should exist");

        vec![
            Button::new(load_button_images(
                &mut assets,
                "UI.nx/Login.img/Title/BtLogin",
            ))
            .with_transform(Transform::from_xyz(454.0, 279.0, 11.0))
            .with_on_click(|| log::info!("login")),
            // TODO: is this supposed to be a checkbox?
            Button::new(load_button_images(
                &mut assets,
                "UI.nx/Login.img/Title/BtLoginIDSave",
            ))
            .with_transform(Transform::from_xyz(303.0, 332.0, 11.0))
            .with_on_click(|| log::info!("save_login_id")),
            Button::new(load_button_images(
                &mut assets,
                "UI.nx/Login.img/Title/BtLoginIDLost",
            ))
            .with_transform(Transform::from_xyz(375.0, 332.0, 11.0))
            .with_on_click(|| log::info!("find_login_id")),
            Button::new(load_button_images(
                &mut assets,
                "UI.nx/Login.img/Title/BtPasswdLost",
            ))
            .with_transform(Transform::from_xyz(447.0, 332.0, 11.0))
            .with_on_click(|| log::info!("find_password")),
            Button::new(load_button_images(
                &mut assets,
                "UI.nx/Login.img/Title/BtNew",
            ))
            .with_transform(Transform::from_xyz(291.0, 352.0, 11.0))
            .with_on_click(|| log::info!("join")),
            Button::new(load_button_images(
                &mut assets,
                "UI.nx/Login.img/Title/BtHomePage",
            ))
            .with_transform(Transform::from_xyz(363.0, 352.0, 11.0))
            .with_on_click(|| log::info!("website")),
            Button::new(load_button_images(
                &mut assets,
                "UI.nx/Login.img/Title/BtQuit",
            ))
            .with_transform(Transform::from_xyz(435.0, 352.0, 11.0))
            .with_on_click(|| std::process::exit(0)),
        ]
    };

    state.buttons.extend(buttons);
}

fn init_text_inputs(state: &mut State) {
    let username_input = TextInput::new(150, 24)
        .with_font(FontDescriptor::new("Arial", 13, Colour::white()))
        .with_transform(Transform::from_xyz(296.0, 279.0, 11.0))
        .with_focused(true)
        .with_max_length(12);

    let password_input = TextInput::new(150, 24)
        .with_font(FontDescriptor::new("Arial", 13, Colour::white()))
        .with_transform(Transform::from_xyz(296.0, 305.0, 11.0))
        .with_masked(true)
        .with_max_length(12);

    state.text_inputs.push(username_input);
    state.text_inputs.push(password_input);
}

fn init_test_map(state: &mut State) {
    let (sprites, map_physics) = {
        let mut assets = state
            .get_resource_mut::<AssetManager>()
            .expect("AssetManager should exist");

        let (specs, map_physics) = collect_test_map_data(&assets);
        let mut sprites = Vec::new();

        for spec in specs {
            let Some(image) = assets.load_image(&spec.path) else {
                log::warn!("Skipping missing map sprite {}", spec.path);
                continue;
            };

            let view_x = if spec.apply_view_offset {
                TEST_MAP_VIEW_X
            } else {
                0.0
            };
            let view_y = if spec.apply_view_offset {
                TEST_MAP_VIEW_Y
            } else {
                0.0
            };

            let sprite = Sprite::new(image).with_transform(Transform::from_xyz(
                spec.x + view_x,
                spec.y + view_y,
                spec.z,
            ));

            sprites.push(if spec.apply_view_offset {
                sprite
            } else {
                sprite.with_screen_space()
            });
        }

        if DEBUG_FOOTHOLDS {
            if let Some(map_physics) = &map_physics {
                append_debug_foothold_sprites(&mut sprites, &mut assets, map_physics);
            }
        }

        (sprites, map_physics)
    };

    state.sprites.extend(sprites);

    if let Some(map_physics) = map_physics {
        state.insert_resource(map_physics);
    }
}

fn init_test_character(state: &mut State) {
    let start_index = state.sprites.len();
    let start = test_character_start_position(state);
    let position = (start.x, start.y);
    let mut parts = Vec::new();

    let sprites = {
        let mut assets = state
            .get_resource_mut::<AssetManager>()
            .expect("AssetManager should exist");

        let placements = test_character_part_placements(&assets, position);
        let mut sprites = Vec::new();

        for placement in placements {
            let Some(image) = assets.load_image(&placement.path) else {
                log::warn!("Skipping missing character sprite {}", placement.path);
                continue;
            };

            parts.push(PlayerPart {
                sprite_index: start_index + sprites.len(),
                offset_x: placement.x - position.0,
                offset_y: placement.y - position.1,
            });

            sprites.push(Sprite::new(image).with_transform(Transform::from_xyz(
                placement.x,
                placement.y,
                placement.z,
            )));
        }

        sprites
    };

    state.sprites.extend(sprites);
    let mut player = Player::new(position.0, position.1, parts);
    if let Some(ground) = start.ground {
        player = player.with_ground(ground.id, ground.layer, ground.slope);
    }
    state.insert_resource(player);
    center_camera_on_test_character(state, position);
}

fn center_camera_on_test_character(state: &State, position: (f32, f32)) {
    let Some(map_physics) = state.get_resource::<MapPhysics>() else {
        return;
    };
    let walls = map_physics.walls();
    let mut borders = map_physics.borders();
    borders.max -= TEST_CAMERA_BOTTOM_PADDING;
    drop(map_physics);

    let Some(mut camera) = state.get_resource_mut::<Camera>() else {
        return;
    };
    camera.center_on_clamped(
        position.0,
        position.1 + TEST_CAMERA_INITIAL_Y_OFFSET,
        walls,
        borders,
    );
}

fn test_character_part_placements(
    assets: &AssetManager,
    position: (f32, f32),
) -> Vec<CharacterPartPlacement> {
    let body_path = "Character.nx/00002000.img/stand1/0/body";
    let arm_path = "Character.nx/00002000.img/stand1/0/arm";
    let head_path = "Character.nx/00012000.img/stand1/0/head";
    let face_path = "Character.nx/Face/00020000.img/default/face";
    let hair_path = "Character.nx/Hair/00030000.img/default/hair";

    let Some(anchors) = character_anchors(assets, body_path, arm_path, head_path, hair_path) else {
        log::warn!("Falling back to approximate character placement");
        return fallback_character_part_placements(position);
    };

    let body = position;
    let arm = add_points(position, sub_points(anchors.body_navel, anchors.arm_navel));
    let head = add_points(position, sub_points(anchors.body_neck, anchors.head_neck));
    let face = add_points(head, anchors.head_brow);
    let hair = add_points(
        position,
        add_points(
            sub_points(anchors.head_brow, anchors.head_neck),
            sub_points(anchors.body_neck, anchors.hair_brow),
        ),
    );

    vec![
        CharacterPartPlacement {
            path: body_path.to_string(),
            x: body.0,
            y: body.1,
            z: 17.0,
        },
        CharacterPartPlacement {
            path: arm_path.to_string(),
            x: arm.0,
            y: arm.1,
            z: 18.0,
        },
        CharacterPartPlacement {
            path: head_path.to_string(),
            x: head.0,
            y: head.1,
            z: 19.0,
        },
        CharacterPartPlacement {
            path: face_path.to_string(),
            x: face.0,
            y: face.1,
            z: 20.0,
        },
        CharacterPartPlacement {
            path: hair_path.to_string(),
            x: hair.0,
            y: hair.1,
            z: 21.0,
        },
    ]
}

struct CharacterAnchors {
    body_navel: (f32, f32),
    body_neck: (f32, f32),
    arm_navel: (f32, f32),
    head_neck: (f32, f32),
    head_brow: (f32, f32),
    hair_brow: (f32, f32),
}

fn character_anchors(
    assets: &AssetManager,
    body_path: &str,
    arm_path: &str,
    head_path: &str,
    hair_path: &str,
) -> Option<CharacterAnchors> {
    let body_navel = assets.with_node(body_path, |node| node_vector(node, "map/navel"))??;
    let body_neck = assets.with_node(body_path, |node| node_vector(node, "map/neck"))??;
    let arm_navel = assets.with_node(arm_path, |node| node_vector(node, "map/navel"))??;
    let head_neck = assets.with_node(head_path, |node| node_vector(node, "map/neck"))??;
    let head_brow = assets.with_node(head_path, |node| node_vector(node, "map/brow"))??;
    let hair_brow = assets.with_node(hair_path, |node| node_vector(node, "map/brow"))??;

    Some(CharacterAnchors {
        body_navel,
        body_neck,
        arm_navel,
        head_neck,
        head_brow,
        hair_brow,
    })
}

fn fallback_character_part_placements(position: (f32, f32)) -> Vec<CharacterPartPlacement> {
    let body = position;
    let body_navel = add_points(body, (-8.0, -21.0));
    let arm = add_points(body_navel, (13.0, 1.0));

    let head = add_points(position, (-4.0, -47.0));
    let brow = add_points(head, (-4.0, -5.0));
    let face = add_points(brow, (1.0, 12.0));
    let hair = brow;

    vec![
        CharacterPartPlacement {
            path: "Character.nx/00002000.img/stand1/0/body".to_string(),
            x: body.0,
            y: body.1,
            z: 17.0,
        },
        CharacterPartPlacement {
            path: "Character.nx/00002000.img/stand1/0/arm".to_string(),
            x: arm.0,
            y: arm.1,
            z: 18.0,
        },
        CharacterPartPlacement {
            path: "Character.nx/00012000.img/front/head".to_string(),
            x: head.0,
            y: head.1,
            z: 19.0,
        },
        CharacterPartPlacement {
            path: "Character.nx/Face/00020000.img/default/face".to_string(),
            x: face.0,
            y: face.1,
            z: 20.0,
        },
        CharacterPartPlacement {
            path: "Character.nx/Hair/00030000.img/default/hair".to_string(),
            x: hair.0,
            y: hair.1,
            z: 21.0,
        },
    ]
}

fn test_character_start_position(state: &State) -> CharacterStart {
    let ground = state
        .get_resource::<MapPhysics>()
        .and_then(|map_physics| map_physics.ground_below(TEST_CHARACTER_X, TEST_CHARACTER_SPAWN_Y));

    CharacterStart {
        x: TEST_CHARACTER_X,
        y: ground.map(|ground| ground.y).unwrap_or(350.0),
        ground,
    }
}

fn sub_points(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    (a.0 - b.0, a.1 - b.1)
}

fn add_points(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    (a.0 + b.0, a.1 + b.1)
}

fn collect_test_map_data(assets: &AssetManager) -> (Vec<MapSpritePlacement>, Option<MapPhysics>) {
    let map_path = map_node_path(TEST_MAP_ID);
    let Some(data) = assets.with_node(&map_path, |map| {
        let mut specs = Vec::new();
        collect_background_specs(map, &mut specs);
        collect_layer_specs(assets, map, &mut specs);
        let map_physics = MapPhysics::from_node(map);
        (specs, map_physics)
    }) else {
        log::warn!("Missing test map {}", map_path);
        return (Vec::new(), None);
    };

    data
}

fn append_debug_foothold_sprites(
    sprites: &mut Vec<Sprite>,
    assets: &mut AssetManager,
    map_physics: &MapPhysics,
) {
    for line in map_physics.debug_foothold_lines() {
        let (x, y, width, height, data) = debug_line_image(line);
        let image = assets.create_image(
            format!("debug/foothold/{}", line.id),
            width,
            height,
            data,
            None,
        );

        sprites.push(Sprite::new(image).with_transform(Transform::from_xyz(
            x,
            y,
            DEBUG_FOOTHOLD_Z,
        )));
    }
}

fn debug_line_image(line: MapLine) -> (f32, f32, u32, u32, Vec<u8>) {
    let min_x = line.x1.min(line.x2).floor();
    let min_y = line.y1.min(line.y2).floor();
    let max_x = line.x1.max(line.x2).ceil();
    let max_y = line.y1.max(line.y2).ceil();
    let width = (max_x - min_x + 1.0).max(1.0) as u32;
    let height = (max_y - min_y + 1.0).max(1.0) as u32;
    let mut data = vec![0; width as usize * height as usize * 4];

    let dx = line.x2 - line.x1;
    let dy = line.y2 - line.y1;
    let steps = dx.abs().max(dy.abs()).max(1.0) as usize;

    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let x = (line.x1 + dx * t - min_x).round() as i32;
        let y = (line.y1 + dy * t - min_y).round() as i32;
        set_debug_pixel(&mut data, width, height, x, y);
        set_debug_pixel(&mut data, width, height, x, y + 1);
    }

    (min_x, min_y, width, height, data)
}

fn set_debug_pixel(data: &mut [u8], width: u32, height: u32, x: i32, y: i32) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }

    let index = (y as u32 * width + x as u32) as usize * 4;
    data[index] = 0;
    data[index + 1] = 0;
    data[index + 2] = 255;
    data[index + 3] = 255;
}

fn collect_background_specs(map: NxNode, specs: &mut Vec<MapSpritePlacement>) {
    let Some(back) = map.get("back") else {
        return;
    };
    let Ok(backgrounds) = back.iter() else {
        return;
    };

    for background in backgrounds {
        let Some(background_set) = node_string(background, "bS") else {
            continue;
        };
        let no = node_integer(background, "no").unwrap_or_default();
        let x = node_integer(background, "x").unwrap_or_default() as f32;
        let y = node_integer(background, "y").unwrap_or_default() as f32;
        let front = node_integer(background, "front").unwrap_or_default() != 0;
        let frame_group = if node_integer(background, "ani").unwrap_or_default() == 0 {
            "back"
        } else {
            "ani"
        };

        specs.push(MapSpritePlacement {
            path: format!("Map.nx/Back/{}.img/{}/{}", background_set, frame_group, no),
            x,
            y,
            z: if front { 19.0 } else { 0.0 },
            apply_view_offset: false,
        });
    }
}

fn collect_layer_specs(assets: &AssetManager, map: NxNode, specs: &mut Vec<MapSpritePlacement>) {
    for layer in 0..8 {
        let Some(layer_node) = map.get(&layer.to_string()) else {
            continue;
        };

        collect_obj_specs(layer_node, layer, specs);
        collect_tile_specs(assets, layer_node, layer, specs);
    }
}

fn collect_obj_specs(layer_node: NxNode, layer: i32, specs: &mut Vec<MapSpritePlacement>) {
    let Some(objs) = layer_node.get("obj") else {
        return;
    };
    let Ok(objs) = objs.iter() else {
        return;
    };

    for obj in objs {
        let Some(object_set) = node_string(obj, "oS") else {
            continue;
        };
        let Some(layer_0) = node_string(obj, "l0") else {
            continue;
        };
        let Some(layer_1) = node_string(obj, "l1") else {
            continue;
        };
        let Some(layer_2) = node_string(obj, "l2") else {
            continue;
        };

        specs.push(MapSpritePlacement {
            path: format!(
                "Map.nx/Obj/{}.img/{}/{}/{}/0",
                object_set, layer_0, layer_1, layer_2
            ),
            x: node_integer(obj, "x").unwrap_or_default() as f32,
            y: node_integer(obj, "y").unwrap_or_default() as f32,
            z: map_object_z(layer, node_integer(obj, "z").unwrap_or_default()),
            apply_view_offset: true,
        });
    }
}

fn collect_tile_specs(
    assets: &AssetManager,
    layer_node: NxNode,
    layer: i32,
    specs: &mut Vec<MapSpritePlacement>,
) {
    let Some(tile_set) = layer_node
        .get("info")
        .and_then(|info| node_string(info, "tS"))
    else {
        return;
    };
    let Some(tiles) = layer_node.get("tile") else {
        return;
    };
    let Ok(tiles) = tiles.iter() else {
        return;
    };

    for tile in tiles {
        let Some(tile_group) = node_string(tile, "u") else {
            continue;
        };
        let no = node_integer(tile, "no").unwrap_or_default();
        let path = format!("Map.nx/Tile/{}.img/{}/{}", tile_set, tile_group, no);
        let tile_z = tile_render_z(assets, &path, tile);

        specs.push(MapSpritePlacement {
            path,
            x: node_integer(tile, "x").unwrap_or_default() as f32,
            y: node_integer(tile, "y").unwrap_or_default() as f32,
            z: map_tile_z(layer, tile_z),
            apply_view_offset: true,
        });
    }
}

fn tile_render_z(assets: &AssetManager, path: &str, tile: NxNode) -> i64 {
    let image_z = assets
        .with_node(path, |image| node_integer(image, "z").unwrap_or_default())
        .unwrap_or_default();

    if image_z == 0 {
        node_integer(tile, "zM").unwrap_or_default()
    } else {
        image_z
    }
}

fn map_node_path(map_id: i32) -> String {
    format!(
        "Map.nx/Map/Map{}/{}.img",
        map_id / 100000000,
        format!("{:09}", map_id)
    )
}

fn map_object_z(layer: i32, z: i64) -> f32 {
    map_layer_base_z(layer) + z as f32 / 1000.0
}

fn map_tile_z(layer: i32, z: i64) -> f32 {
    map_layer_base_z(layer) + 0.5 + z as f32 / 1000.0
}

fn map_layer_base_z(layer: i32) -> f32 {
    1.0 + layer as f32 * 2.0
}

fn node_integer(node: NxNode, child: &str) -> Option<i64> {
    node.get(child).integer().ok().flatten()
}

fn node_vector(node: NxNode, child: &str) -> Option<(f32, f32)> {
    node.get(child)
        .vector()
        .ok()
        .flatten()
        .map(|(x, y)| (x as f32, y as f32))
}

fn node_string(node: NxNode, child: &str) -> Option<String> {
    node.get(child).string().ok().flatten().map(str::to_string)
}

fn init_game_status_bar(state: &mut State) {
    let (sprites, buttons) = {
        let mut assets = state
            .get_resource_mut::<AssetManager>()
            .expect("AssetManager should exist");

        let mut sprites = Vec::new();
        push_sprite(
            &mut sprites,
            &mut assets,
            "UI.nx/StatusBar3.img/mainBar/EXPBar/backgrnd",
            0.0,
            87.0,
            20.0,
        );
        push_sprite(
            &mut sprites,
            &mut assets,
            "UI.nx/StatusBar3.img/mainBar/EXPBar/800/layer:back",
            0.0,
            87.0,
            21.0,
        );
        push_sprite(
            &mut sprites,
            &mut assets,
            "UI.nx/StatusBar3.img/mainBar/EXPBar/800/layer:gauge",
            0.0,
            87.0,
            22.0,
        );
        push_sprite(
            &mut sprites,
            &mut assets,
            "UI.nx/StatusBar3.img/mainBar/EXPBar/800/layer:cover",
            0.0,
            87.0,
            23.0,
        );

        let hpmp_x = 412.0;
        let hpmp_y = 40.0;
        push_sprite(
            &mut sprites,
            &mut assets,
            "UI.nx/StatusBar3.img/mainBar/status800/backgrnd",
            hpmp_x - 1.0,
            hpmp_y,
            24.0,
        );
        push_sprite(
            &mut sprites,
            &mut assets,
            "UI.nx/StatusBar3.img/mainBar/status800/gauge/hp/layer:0",
            hpmp_x,
            hpmp_y,
            25.0,
        );
        push_sprite(
            &mut sprites,
            &mut assets,
            "UI.nx/StatusBar3.img/mainBar/status800/gauge/mp/layer:0",
            hpmp_x,
            hpmp_y,
            26.0,
        );
        push_sprite(
            &mut sprites,
            &mut assets,
            "UI.nx/StatusBar3.img/mainBar/status800/layer:cover",
            hpmp_x - 1.0,
            hpmp_y,
            27.0,
        );
        push_sprite(
            &mut sprites,
            &mut assets,
            "UI.nx/StatusBar3.img/mainBar/status800/layer:Lv",
            hpmp_x,
            hpmp_y,
            28.0,
        );

        push_sprite(
            &mut sprites,
            &mut assets,
            "UI.nx/StatusBar3.img/mainBar/quickSlot/backgrnd",
            579.0,
            0.0,
            24.0,
        );
        push_sprite(
            &mut sprites,
            &mut assets,
            "UI.nx/StatusBar3.img/mainBar/quickSlot/layer:cover",
            579.0,
            0.0,
            25.0,
        );

        let button_pos = status_bar_transform(591.0, 73.0, 30.0);
        let mut buttons = Vec::new();
        push_button(
            &mut buttons,
            &mut assets,
            "UI.nx/StatusBar3.img/mainBar/menu/button:CashShop",
            button_pos,
            || log::info!("cash shop"),
        );
        push_button(
            &mut buttons,
            &mut assets,
            "UI.nx/StatusBar3.img/mainBar/menu/button:Menu",
            button_pos,
            || log::info!("menu"),
        );
        push_button(
            &mut buttons,
            &mut assets,
            "UI.nx/StatusBar3.img/mainBar/menu/button:Setting",
            button_pos,
            || log::info!("settings"),
        );
        push_button(
            &mut buttons,
            &mut assets,
            "UI.nx/StatusBar3.img/mainBar/menu/button:Character",
            button_pos,
            || log::info!("character"),
        );
        push_button(
            &mut buttons,
            &mut assets,
            "UI.nx/StatusBar3.img/mainBar/menu/button:Community",
            button_pos,
            || log::info!("community"),
        );
        push_button(
            &mut buttons,
            &mut assets,
            "UI.nx/StatusBar3.img/mainBar/menu/button:Event",
            button_pos,
            || log::info!("event"),
        );

        (sprites, buttons)
    };

    state.sprites.extend(sprites);
    state.buttons.extend(buttons);
}

fn push_sprite(
    sprites: &mut Vec<Sprite>,
    assets: &mut AssetManager,
    path: &str,
    x: f32,
    y: f32,
    z: f32,
) {
    let Some(image) = assets.load_image(path) else {
        log::warn!("Skipping missing game scene sprite {}", path);
        return;
    };

    sprites.push(
        Sprite::new(image)
            .with_transform(status_bar_transform(x, y, z))
            .with_screen_space(),
    );
}

fn status_bar_transform(x: f32, y: f32, z: f32) -> Transform {
    Transform::from_xyz(x, GAME_STATUS_BAR_Y + y, z)
}

fn push_button(
    buttons: &mut Vec<Button>,
    assets: &mut AssetManager,
    path: &str,
    transform: Transform,
    on_click: fn(),
) {
    let images = load_button_images(assets, path);

    if images[ButtonState::Default as usize].is_none() {
        log::warn!("Skipping missing game scene button {}", path);
        return;
    }

    buttons.push(
        Button::new(images)
            .with_transform(transform)
            .with_on_click(on_click),
    );
}
