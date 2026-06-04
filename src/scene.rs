use crate::{
    component::{Colour, Transform},
    graphics::{
        ui::{load_button_images, Button, ButtonState, TextInput},
        Sprite,
    },
    resource::{AssetManager, FontDescriptor},
    state::State,
};

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
        init_game_status_bar(state);
    }
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

        let button_pos = Transform::from_xyz(591.0, 73.0, 30.0);
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

    sprites.push(Sprite::new(image).with_transform(Transform::from_xyz(x, y, z)));
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
