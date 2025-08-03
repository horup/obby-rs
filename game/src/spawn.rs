use glam::Vec2;

use crate::{update, Clip, Entity, EntityVariant, Game};

pub fn spawn_player(game:&mut Game, pos:Vec2) -> &mut Entity {
    let skin = game.skin_chosen;
    let e = game.spawn_entity();
    e.is_player = true;
    e.pos = pos;
    e.pos_start = e.pos;
    e.update = update::update_player_starting;
    e.variant = EntityVariant::Player {
        skin
    };
    e.timer0.start(1.0);
    e
}

pub fn spawn_coin(game:&mut Game, pos:Vec2) -> &mut Entity {
    let e = game.spawn_entity();
    e.pos = pos;
    e.variant = EntityVariant::Coin;
    e.pos_start = e.pos;
    e.update = update::update_coin;
    e.timer0.timer_start_sec = 2.0;
    e.clip = Clip::NoClip;
    e
}

pub fn spawn_goal(game:&mut Game, pos:Vec2) -> &mut Entity {
    let e = game.spawn_entity();
    e.is_goal = true;
    e.pos = pos;
    e.pos_start = e.pos;
    e.variant = EntityVariant::Goal;
    e
}

pub fn spawn_cloud(game:&mut Game, pos:Vec2) -> &mut Entity {
    let e = game.spawn_entity();
    e.pos = pos;
    e.pos_start = e.pos;
    e.variant = EntityVariant::Cloud;
    e.update = update::update_cloud;
    e
}