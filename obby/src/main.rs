mod assets;
use crate::assets::{Assets, Atlas};
use game::{Context as _, Game};
use gamepads::Gamepads;
use macroquad::{
    audio::{PlaySoundParams, load_sound, play_sound},
    miniquad::window::set_window_size,
    prelude::*,
    rand::rand,
};
use std::collections::HashMap;

const CHARACTERS: u8 = 5;

pub struct Context {
    pub map_list: Vec<String>,
    pub assets: Assets,
    pub frame_time: f32,
    pub d_pad: Vec2,
    pub keys_down: HashMap<game::Keys, ()>,
    pub keys_pressed: HashMap<game::Keys, ()>,
    pub background: Color,
}

impl Context {
    pub fn play_sound(&self, key: &str, looped: bool, volume: f32) {
        let Some(sounds) = self.assets.sfx.get(key) else {
            return;
        };
        let i = rand() % sounds.len() as u32;
        let Some(sound) = sounds.get(i as usize) else {
            return;
        };
        play_sound(
            sound,
            PlaySoundParams {
                looped,
                volume,
            },
        );
    }
}

#[derive(Clone)]
pub struct Map {
    pub tiled_map: tiled::Map,
}

impl game::Map for Map {
    fn width(&self) -> u32 {
        self.tiled_map.width
    }

    fn height(&self) -> u32 {
        self.tiled_map.height
    }

    fn tile(&self, x: i32, y: i32) -> game::MapTile {
        let mut game_tile = game::MapTile::default();
        let layer = self
            .tiled_map
            .get_layer(0)
            .and_then(|l| l.as_tile_layer())
            .and_then(|l| l.get_tile(x, y));
        if let Some(tile) = layer {
            game_tile.variant = tile.id();
            if let Some(tile_data) = tile.get_tile() {
                if let Some(user_type) = &tile_data.user_type {
                    for user_type in user_type.split_whitespace() {
                        match user_type {
                            "block" => game_tile.is_block = true,
                            "player" => game_tile.is_player = true,
                            "goal" => game_tile.is_goal = true,
                            "foreground" => game_tile.is_foreground = true,
                            "entity" => game_tile.is_entity = true,
                            "coin" => game_tile.is_coin = true,
                            "deadly" => game_tile.is_deadly = true,
                            "cloud" => game_tile.is_cloud = true,
                            _ => {}
                        }
                    }
                }
            }
        }
        game_tile
    }

    fn background(&self) -> (u8, u8, u8) {
        self.tiled_map
            .background_color
            .map(|bg| (bg.red, bg.green, bg.blue))
            .unwrap_or((0, 0, 0))
    }
}

impl game::Context for Context {
    fn map(&mut self, name: &str) -> game::MapResult {
        self.assets.load_map(name)
    }

    fn dt(&self) -> f32 {
        self.frame_time
    }

    fn is_key_down(&self, key: game::Keys) -> bool {
        self.keys_down.contains_key(&key)
    }

    fn d_pad(&self) -> Vec2 {
        self.d_pad
    }

    fn map_list(&self) -> &Vec<String> {
        &self.map_list
    }

    fn rand_f32(&self) -> f32 {
        rand() as f32 / u32::MAX as f32
    }

    fn is_key_pressed(&self, key: game::Keys) -> bool {
        self.keys_pressed.contains_key(&key)
    }

    fn is_any_key_pressed(&self) -> bool {
        !self.keys_pressed.is_empty()
    }
}

impl Context {
    pub async fn new() -> Self {
        let tileset = assets::Atlas::new(
            20,
            20,
            load_texture("res/imgs/tileset.png")
                .await
                .expect("failed to load tileset"),
        );

        let map_list: Vec<String> = load_string("res/maps.txt")
            .await
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect();

        let mut sfx = HashMap::default();

        for line in load_string("res/sfx.csv")
            .await
            .expect("failed to load sfx.csv")
            .lines()
        {
            let mut cols = line.split(",");
            let key = cols.next().unwrap();
            let mut sounds = Vec::default();
            for path in cols {
                let path = path.trim();
                let sound = load_sound(path).await.expect("failed to load sound");
                sounds.push(sound);
            }
            sfx.insert(key.to_string(), sounds);
        }

        let mut sound_coin = Vec::new();
        for n in 1..=2 {
            sound_coin.push(load_sound(&format!("res/sfx/coin{n}.wav")).await.unwrap());
        }
        Context {
            map_list,
            assets: Assets {
                maps: Default::default(),
                maps_pending: Default::default(),
                tileset,
                sfx,
            },
            frame_time: 0.0,
            d_pad: Default::default(),
            keys_down: Default::default(),
            keys_pressed: Default::default(),
            background: WHITE,
        }
    }
}

fn process_events(app_state: &mut AppState, game: &Game, ctx: &mut Context) {
    for event in game.events.iter() {
        match event {
            game::Event::PickupCoin => {
                ctx.play_sound("coin", false, 1.0);
            }
            game::Event::Won => {
                ctx.play_sound("win", false, 1.0);
            }
            game::Event::Died => {
                ctx.play_sound("lost", false, 1.0);
            }
            game::Event::PickupExtraLife => {
                ctx.play_sound("extra_life", false, 1.0);
            }
            game::Event::PlayerJump => {
                ctx.play_sound("jump", false, 1.0);
            }
            game::Event::GameOver { score } => {
                *app_state = AppState::GameOver {
                    score: *score as f32,
                    score_display: 0.0,
                };
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum AppState {
    Title { elapsed: f32 },
    Game,
    GameOver { score: f32, score_display: f32 },
    CharacterSelection { selection: u8 },
}

#[macroquad::main("HØRUP'S OBBY")]
async fn main() {
    let scale = 2;
    let target_width = 768 * scale;
    let target_height = 512 * scale;
    set_window_size(target_width, target_height);

    let render_target = render_target(target_width, target_height);
    render_target.texture.set_filter(FilterMode::Linear);

    let camera = Camera2D {
        render_target: Some(render_target.clone()),
        target: vec2(target_width as f32 / 2.0, target_height as f32 / 2.0),
        zoom: vec2(2.0 / target_width as f32, 2.0 / target_height as f32),
        ..Default::default()
    };

    let mut ctx = Context::new().await;

    let mut game = Game::default();
    game.init(&mut ctx);

    let target_width = target_width as f32;
    let target_height = target_height as f32;
    let mut camera_offset_x_px = 0.0;

    let mut gamepads = Gamepads::new();
    let mut full_screen = true;

    let mut app_state = AppState::Title { elapsed: 0.0 };
    let mut secs = 0.0;
    loop {
        let flashing = (secs * 3.0) as i32 % 2 == 0;
        if is_key_down(KeyCode::LeftAlt) && is_key_pressed(KeyCode::Enter) {
            set_fullscreen(full_screen);
            full_screen = !full_screen;
        }
        if is_key_pressed(KeyCode::F1) {
            if app_state != AppState::Game {
                app_state = AppState::Game;
            } else {
                game.next_level(&mut ctx);
            }
        }
        if is_key_pressed(KeyCode::F2) {
            app_state = AppState::CharacterSelection { selection: 0 };
        }
        set_camera(&camera);
        ctx.frame_time = get_frame_time().min(0.1);
        ctx.keys_down.clear();
        ctx.keys_pressed.clear();
        // keyboard input
        if is_key_down(KeyCode::Space) || is_key_down(KeyCode::Up) {
            ctx.keys_down.insert(game::Keys::Space, ());
        }
        if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Up) {
            ctx.keys_pressed.insert(game::Keys::Space, ());
        }

        let mut d_pad = Vec2::default();
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            d_pad.x = -1.0;
            ctx.keys_down.insert(game::Keys::Left, ());
        } else if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            d_pad.x = 1.0;
            ctx.keys_down.insert(game::Keys::Right, ());
        }
        if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
            d_pad.x = -1.0;
            ctx.keys_pressed.insert(game::Keys::Left, ());
        } else if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
            d_pad.x = 1.0;
            ctx.keys_pressed.insert(game::Keys::Right, ());
        }

        // gamepad input
        gamepads.poll();

        for gamepad in gamepads.all() {
            for button in gamepad.all_currently_pressed() {
                //println!("Pressed button: {:?}", button);
                match button {
                    gamepads::Button::DPadLeft => {
                        d_pad.x = -1.0;
                        ctx.keys_down.insert(game::Keys::Left, ());
                    }
                    gamepads::Button::DPadRight => {
                        d_pad.x = 1.0;
                        ctx.keys_down.insert(game::Keys::Right, ());
                    }
                    gamepads::Button::ActionRight => {
                        ctx.keys_down.insert(game::Keys::Space, ());
                    }
                    gamepads::Button::ActionDown => {
                        ctx.keys_down.insert(game::Keys::Space, ());
                    }
                    _ => {}
                }
            }

            for button in gamepad.all_just_pressed() {
                match button {
                    gamepads::Button::ActionRight => {
                        ctx.keys_pressed.insert(game::Keys::Space, ());
                    }
                    gamepads::Button::ActionDown => {
                        ctx.keys_pressed.insert(game::Keys::Space, ());
                    }
                    gamepads::Button::DPadLeft => {
                        ctx.keys_pressed.insert(game::Keys::Left, ());
                    }
                    gamepads::Button::DPadRight => {
                        ctx.keys_pressed.insert(game::Keys::Right, ());
                    }
                    _ => {}
                }
            }
        }

        ctx.d_pad = d_pad.normalize_or_zero();

        // update
        match &mut app_state {
            AppState::Title { elapsed } => {
                *elapsed += ctx.dt();
                if *elapsed > 1.0
                    && ctx.is_any_key_pressed() {
                        app_state = AppState::CharacterSelection { selection: 0 };
                    }
            }
            AppState::Game => {
                game.update(&mut ctx);
                if is_key_pressed(KeyCode::F2) {
                    game.events.push(game::Event::GameOver { score: 1337 });
                }
                process_events(&mut app_state, &game, &mut ctx);
                game.events.clear();
            }
            AppState::GameOver {
                score,
                score_display,
            } => {
                *score_display += ctx.dt() * *score / 4.0;
                if score_display >= score {
                    *score_display = *score;
                    if ctx.is_any_key_pressed() {
                        app_state = AppState::Title { elapsed: 0.0 };
                    }
                }
            }
            AppState::CharacterSelection { selection } => {
                if ctx.is_key_pressed(game::Keys::Space) {
                    game = Game::default();
                    game.skin_chosen = *selection as u32;
                    game.init(&mut ctx);
                    app_state = AppState::Game;
                } else {
                    if ctx.is_key_pressed(game::Keys::Right) {
                        *selection = selection.wrapping_add(1);
                    } else if ctx.is_key_pressed(game::Keys::Left) {
                        *selection = selection.wrapping_sub(1);
                    }
                    if *selection >= CHARACTERS {
                        *selection = 0;
                    }
                }
            }
        }

        // draw
        match &app_state {
            AppState::Title { elapsed } => {
                let flashing = if *elapsed > 1.0 { flashing } else { false };
                draw_title(target_width, target_height, flashing);
            }
            AppState::Game => {
                draw_game(
                    &game,
                    &mut camera_offset_x_px,
                    target_width,
                    target_height,
                    &mut ctx,
                );
            }
            AppState::GameOver {
                score,
                score_display,
            } => {
                let flashing = if score == score_display {
                    flashing
                } else {
                    false
                };
                draw_gameover(target_width, target_height, flashing, *score_display as u32);
            }
            AppState::CharacterSelection { selection } => {
                draw_character_selection(
                    target_width,
                    target_height,
                    selection,
                    &ctx.assets.tileset,
                );
            }
        }
        // Blit target to screen
        set_default_camera();
        blit_render_target(&render_target.texture, target_width, target_height);

        ctx.assets.load_pending().await;
        next_frame().await;
        secs += get_frame_time();
    }
}

const SKIN_INDEX: [f32; 5] = [120.0, 121.0, 122.0, 123.0, 124.0];

fn draw_character_selection(
    target_width: f32,
    target_height: f32,
    selection: &u8,
    tileset: &Atlas,
) {
    clear_background(BLACK);
    let m = target_width / (CHARACTERS + 1) as f32;

    let font_size = target_height / 8.0;
    let s = "CHOOSE CHARACTER".to_string();
    let measure = measure_text(&s, None, font_size as u16, 1.0);
    let x = target_width / 2.0 - measure.width / 2.0;
    let y = target_height / 4.0 - measure.height;
    draw_text(&s, x, y, font_size, WHITE);

    let texts = ["WILLIAM", "VIKTOR", "SIGGA", "LOUISE", "SØREN"];

    let draw_character = |col: u8, highlighted: bool| {
        let color = if highlighted { WHITE } else { DARKGRAY };

        let size = m;
        let x = m * col as f32 + m / 2.0;
        let y = target_height / 2.0 - size / 2.0;
        let index = SKIN_INDEX[col as usize];
        draw_atlas(
            tileset,
            x,
            y,
            index,
            color,
            (size, size).into(),
            false,
            false,
        );
        let text = texts[col as usize];

        let font_size = target_height / 16.0;
        let measure = measure_text(text, None, font_size as u16, 1.0);
        let x = x + size / 2.0 - measure.width / 2.0;
        let y = y + size + measure.height * 2.0;
        draw_text(text, x, y, font_size, color);
    };

    for i in 0..CHARACTERS {
        draw_character(i, *selection == i);
    }
}

fn draw_gameover(target_width: f32, target_height: f32, flashing: bool, score: u32) {
    clear_background(BLACK);
    let font_size = target_height / 8.0;
    let s = "GAME OVER";
    let measure = measure_text(s, None, font_size as u16, 1.0);
    let x = target_width / 2.0 - measure.width / 2.0;
    let y = target_height / 2.0 - measure.height;
    draw_text(s, x, y, font_size, WHITE);

    let font_size = target_height / 16.0;
    let s = format!("FINAL SCORE:{score}");
    let measure = measure_text(&s, None, font_size as u16, 1.0);
    let x = target_width / 2.0 - measure.width / 2.0;
    let y = target_height / 2.0 - measure.height + font_size / 2.0;
    draw_text(&s, x, y, font_size, WHITE);

    if flashing {
        let font_size = target_height / 16.0;
        let s = "PRESS ANY BUTTON TO CONTINUE";
        let measure = measure_text(s, None, font_size as u16, 1.0);
        let x = target_width / 2.0 - measure.width / 2.0;
        let y = target_height / 2.0 - measure.height + font_size * 3.0;
        draw_text(s, x, y, font_size, RED);
    }
}

fn draw_title(target_width: f32, target_height: f32, flashing: bool) {
    clear_background(BLACK);
    let font_size = target_height / 8.0;
    let s = "HØRUP'S OBBY";
    let measure = measure_text(s, None, font_size as u16, 1.0);
    let x = target_width / 2.0 - measure.width / 2.0;
    let y = target_height / 2.0 - measure.height;
    draw_text(s, x, y, font_size, WHITE);

    let font_size = target_height / 16.0;
    let s = "A GAME ABOUT NOT DYING";
    let measure = measure_text(s, None, font_size as u16, 1.0);
    let x = target_width / 2.0 - measure.width / 2.0;
    let y = target_height / 2.0 - measure.height + font_size / 2.0;
    draw_text(s, x, y, font_size, WHITE);

    if flashing {
        let font_size = target_height / 16.0;
        let s = "PRESS ANY BUTTON TO CONTINUE";
        let measure = measure_text(s, None, font_size as u16, 1.0);
        let x = target_width / 2.0 - measure.width / 2.0;
        let y = target_height / 2.0 - measure.height + font_size * 3.0;
        draw_text(s, x, y, font_size, RED);
    }
}

fn draw_game(
    game: &Game,
    camera_offset_x_px: &mut f32,
    target_width: f32,
    target_height: f32,
    ctx: &mut Context,
) {
    let background_color = game
        .map_current
        .as_ref()
        .map(|x| x.background())
        .unwrap_or_default();

    let grid_height = game.grid_height;
    let grid_width = game.grid_width;
    let cell_size_px = target_height / grid_height as f32;
    let grid_width_px = grid_width as f32 * cell_size_px;

    let player_pos = game
        .entities
        .get(&game.player)
        .map(|e| e.pos)
        .unwrap_or_default();
    let player_pos_px = player_pos.x * cell_size_px;
    *camera_offset_x_px = player_pos_px - target_width / 2.0;

    if target_width < grid_width_px {
        *camera_offset_x_px = camera_offset_x_px.clamp(0.0, grid_width_px - target_width);
    }

    clear_background(Color::from_rgba(
        background_color.0,
        background_color.1,
        background_color.2,
        255,
    ));

    draw_grid(
        game,
        &ctx.assets.tileset,
        *camera_offset_x_px,
        cell_size_px,
        target_width,
        false,
    );

    // Draw entities
    for e in game.entities.values() {
        let x = (e.pos.x - 0.5) * cell_size_px - *camera_offset_x_px;
        let y = (e.pos.y - 0.5) * cell_size_px;
        let index = match e.variant {
            game::EntityVariant::Unknown => 1.0,
            game::EntityVariant::Player { skin } => SKIN_INDEX[skin as usize],
            game::EntityVariant::Goal => 2.0,
            game::EntityVariant::Coin => 21.0,
            game::EntityVariant::Cloud => 81.0,
        };
        let flip_x = matches!(e.dir_x, game::DirX::Left);
        draw_atlas(
            &ctx.assets.tileset,
            x,
            y,
            index,
            WHITE,
            Vec2::new(cell_size_px, cell_size_px),
            flip_x,
            false,
        );
    }

    draw_grid(
        game,
        &ctx.assets.tileset,
        *camera_offset_x_px,
        cell_size_px,
        target_width,
        true,
    );

    draw_hud(game, target_width, target_height, ctx);
}

fn draw_hud(game: &Game, target_width: f32, target_height: f32, ctx: &mut Context) {
    let font_size = target_height / 8.0;
    let margin = 16.0;
    let transparent_color = Color::from_rgba(0, 0, 0, 255 / 4 * 2);
    if !game.center_text.is_empty() {
        let measure = measure_text(&game.center_text, None, font_size as u16, 1.0);
        let x = target_width / 2.0 - measure.width / 2.0;
        let y = target_height / 2.0 - measure.height;
        let margin = 16.0;
        draw_rectangle(
            0.0,
            y - margin,
            target_width,
            measure.height + margin * 2.0,
            transparent_color,
        );
        draw_text_ex(
            &game.center_text,
            x,
            y + measure.height,
            TextParams {
                font_size: font_size as u16,
                ..Default::default()
            },
        );
    }

    let font_size = target_height / 16.0;

    // draw top bar
    draw_rectangle(0.0, 0.0, target_width, font_size, transparent_color);

    // draw SCORE
    let score = game.score;
    let s = &format!("SCORE: {score}");
    let measure = measure_text(s, None, font_size as u16, 1.0);
    let x = margin;
    let y = margin + measure.height;
    draw_text_ex(
        s,
        x,
        y,
        TextParams {
            font_size: font_size as u16,
            ..Default::default()
        },
    );

    // draw LIVES
    let s = &format!("LIVES: {}", game.lives_extra);
    let measure = measure_text(s, None, font_size as u16, 1.0);
    let x = target_width / 3.0 - measure.width / 2.0;
    let y = margin + measure.height;
    draw_text_ex(
        s,
        x,
        y,
        TextParams {
            font_size: font_size as u16,
            ..Default::default()
        },
    );

    // draw LIVES
    let s = &format!("COINS: {}", game.coins);
    let measure = measure_text(s, None, font_size as u16, 1.0);
    let x = target_width / 2.0; // - measure.width / 2.0;
    let y = margin + measure.height;
    draw_text_ex(
        s,
        x,
        y,
        TextParams {
            font_size: font_size as u16,
            ..Default::default()
        },
    );

    // draw level x OF 6
    let s = &format!("LEVEL {} OF {}", game.level_current + 1, ctx.map_list.len());
    let measure = measure_text(s, None, font_size as u16, 1.0);
    let x = target_width - measure.width - margin;
    let y = margin + measure.height;
    draw_text_ex(
        s,
        x,
        y,
        TextParams {
            font_size: font_size as u16,
            ..Default::default()
        },
    );
}

fn draw_grid(
    game: &Game,
    atlas: &Atlas,
    camera_offset_x_px: f32,
    cell_size_px: f32,
    target_width: f32,
    is_foreground: bool,
) {
    let start_x = (camera_offset_x_px / cell_size_px) as u32;
    let end_x = ((camera_offset_x_px + target_width) / cell_size_px) as u32 + 1;
    for y in 0..game.grid_height {
        for x in start_x..end_x {
            let x_px = x as f32 * cell_size_px - camera_offset_x_px;
            let y_px = y as f32 * cell_size_px;
            if let Some(cell) = game.grid.get((x as i32, y as i32)) {
                if cell.is_foreground == is_foreground {
                    draw_atlas(
                        atlas,
                        x_px,
                        y_px,
                        cell.variant as f32,
                        WHITE,
                        Vec2::new(cell_size_px, cell_size_px),
                        false,
                        false,
                    );
                }
            }
        }
    }
}

fn blit_render_target(texture: &Texture2D, target_width: f32, target_height: f32) {
    let size = vec2(target_width, target_height);
    let aspect = size.x / size.y;
    if screen_width() / aspect > screen_height() {
        let size = vec2(screen_height() * aspect, screen_height());
        let x = (screen_width() - size.x) / 2.0;
        draw_texture_ex(
            texture,
            x,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );
    } else {
        let size = vec2(screen_width(), screen_width() / aspect);
        let y = (screen_height() - size.y) / 2.0;
        draw_texture_ex(
            texture,
            0.0,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );
    }
}

fn draw_atlas(
    atlas: &Atlas,
    x: f32,
    y: f32,
    index: f32,
    color: Color,
    dest_size: Vec2,
    flip_x: bool,
    flip_y: bool,
) {
    let source = atlas.index(index);
    draw_texture_ex(
        &atlas.texture,
        x.floor(),
        y.floor(),
        color,
        DrawTextureParams {
            dest_size: Some(dest_size),
            source: Some(source),
            flip_x,
            flip_y,
            ..Default::default()
        },
    );
}
