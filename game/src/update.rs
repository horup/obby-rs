use crate::{Body, Context, DirX, Entity, Event, Game, Keys};
use cliplib::ClipMoveResult;
use glam::Vec2;
use std::{cell::RefCell, f32::consts::PI};

fn apply_gravity(e: &mut Entity, dt: f32) {
    e.vel.y += 60.0 * dt;
}

fn apply_velocity(
    e: &mut Entity,
    game: &mut Game,
    ctx: &mut dyn Context,
    vel: Vec2,
    mut touch: impl FnMut(&Body),
) {
    let dt = ctx.dt();
    let cell = e.cell();
    let other_bodies = game.bodies(cell);

    let vel_y = vel * Vec2::new(0.0, 1.0);
    let vel_x = vel * Vec2::new(1.0, 0.0);

    // vertical movement
    e.is_touching_floor = false;
    let res = cliplib::clip_move(&Body::Entity(e), vel_y * dt, || other_bodies.iter());
    match res {
        ClipMoveResult::Unhindered { new_pos } => e.pos = new_pos,
        ClipMoveResult::Clipped {
            other_body,
            new_pos,
            normal,
        } => {
            touch(other_body);
            e.pos = new_pos;
            //if normal.x == 0.0 {
            if normal.y < 0.0 {
                e.is_touching_floor = true;
            }
            e.vel.y = 0.0;
            //}
        }
    }

    // horizontal movement
    let res = cliplib::clip_move(&Body::Entity(e), vel_x * dt, || other_bodies.iter());
    match res {
        ClipMoveResult::Unhindered { new_pos } => e.pos = new_pos,
        ClipMoveResult::Clipped {
            other_body,
            new_pos,
            ..
        } => {
            touch(other_body);
            e.pos = new_pos;
        }
    }
}

pub fn update_coin(e: &mut Entity, game: &mut Game, ctx: &mut dyn Context) {
    let _ = ctx;
    if game.pause {
        return;
    }
    let d = 1.0 / 8.0;
    let a = f32::sin(game.elapsed_total_sec * PI * 2.0);
    e.pos.y = e.pos_start.y + a * d;

    if let Some(player) = game.entities.get(&game.player) {
        let v = player.pos - e.pos;
        if v.length() < 1.0 {
            e.delete_me = true;
            game.events.push(Event::PickupCoin);
            game.score += 100;
            game.coins += 1;
            if game.coins >= 100 {
                game.coins = 0;
                game.lives_extra += 1;
                game.events.push(Event::PickupExtraLife);
            }
        }
    }
}

pub fn update_player_starting(e: &mut Entity, game: &mut Game, ctx: &mut dyn Context) {
    game.pause = true;
    game.center_text = format!("LEVEL {}", game.level_current + 1);

    if e.timer0.tick(ctx.dt()) {
        game.center_text.clear();
        e.update = update_player;
        game.pause = false;
    }
}

pub fn update_player(e: &mut Entity, game: &mut Game, ctx: &mut dyn Context) {
    game.player = e.id;
    apply_gravity(e, ctx.dt());
    let d_pad = ctx.d_pad();
    let move_speed = 8.0;
    let jump_speed = 20.0;
    let drag_speed = 20.0;

    if ctx.is_key_pressed(Keys::Space)
        && e.is_touching_floor {
            e.vel.y = -jump_speed;
            game.events.push(Event::PlayerJump);
        }
    if !ctx.is_key_down(Keys::Space)
        && e.vel.y < 0.0 {
            e.vel.y = 0.0;
        }

    let dx = d_pad.x * move_speed * ctx.dt() * 16.0;
    if d_pad.x < 0.0 {
        e.dir_x = DirX::Left;
        if e.vel.x > -move_speed {
            e.vel.x += dx;
        }
        if e.vel.x < -move_speed {
            e.vel.x = -move_speed;
        }
    } else if d_pad.x > 0.0 {
        e.dir_x = DirX::Right;
        if e.vel.x < move_speed {
            e.vel.x += dx;
        }
        if e.vel.x > move_speed {
            e.vel.x = move_speed;
        }
    } else {
        let s = e.vel.x.abs() * ctx.dt() * drag_speed;
        if e.vel.x > 0.0 {
            e.vel.x -= s;
            if e.vel.x < 0.0 {
                e.vel.x = 0.0;
            }
        } else if e.vel.x < 0.0 {
            e.vel.x += s;
            if e.vel.x > 0.0 {
                e.vel.x = 0.0;
            }
        }
    }

    let mut dead = false;
    let goal_touched = RefCell::new(false);
    apply_velocity(e, game, ctx, e.vel, |other_body| match other_body {
        Body::Entity(entity) => {
            if entity.is_goal {
                *goal_touched.borrow_mut() = true;
            }
        }
        Body::Block(_i, cell) => {
            if cell.is_deadly {
                dead = true;
            }
        }
        Body::Void(_) => {}
    });

    if *goal_touched.borrow() {
        // won!
        game.center_text = "YOU WON!".to_string();
        e.update = update_player_won;
        e.timer0.start(2.0);
        game.events.push(Event::Won);
        game.score += 1000 * (game.level_current + 1);
        return;
    }
    if e.pos.y > game.grid_height as f32 + 1.0 {
        dead = true;
    }

    if dead {
        // died!
        game.center_text = "YOU DIED!".to_string();
        e.update = update_player_dead;
        e.timer0.start(2.0);
        game.events.push(Event::Died);
    }
}

pub fn update_player_dead(e: &mut Entity, game: &mut Game, ctx: &mut dyn Context) {
    game.pause = true;
    if e.timer0.tick(ctx.dt()) {
        e.delete_me = true;
        let whole_game = game.lives_extra == 0;
        let score = game.score;
        game.restart(ctx, whole_game);
        if !whole_game {
            game.lives_extra -= 1;
        } else {
            game.events.push(Event::GameOver { score });
        }
    }
}

pub fn update_player_won(e: &mut Entity, game: &mut Game, ctx: &mut dyn Context) {
    game.pause = true;
    if e.timer0.tick(ctx.dt()) {
        game.next_level(ctx);
    }
}

pub fn update_cloud(e: &mut Entity, game: &mut Game, ctx: &mut dyn Context) {
    let player_standing_on_cloud = if let Some(player) = game.entities.get(&game.player) {
        let v = player.pos - e.pos;
        v.length() < 1.0 && player.is_touching_floor
    } else {
        false
    };

    let cloud_is_gone = e.pos_start != e.pos;
    let cloud_gone_sec = 0.5;
    let cloud_reappear_sec = 1.0;
    if cloud_is_gone {
        if e.timer0.tick(ctx.dt()) {
            e.pos = e.pos_start;
            e.timer0.start(cloud_gone_sec);
        }
    } else if player_standing_on_cloud {
        if e.timer0.tick(ctx.dt()) {
            e.pos = Vec2::new(-2.0, -2.0);
            e.timer0.start(cloud_reappear_sec);
        }
    } else {
        e.timer0.start(cloud_gone_sec);
    }
}
