use glam::Vec2;

use crate::{Body, Context, Game, Timer};

#[derive(Debug)]
pub struct Entity {
    pub id: u32,
    pub pos: Vec2,
    pub pos_start: Vec2,
    pub vel: Vec2,
    pub is_touching_floor: bool,
    pub is_player: bool,
    pub is_goal: bool,
    pub update: fn(&mut Entity, &mut Game, &mut dyn Context),
    pub delete_me: bool,
    pub timer0: Timer,
    pub dir_x: DirX,
    pub variant:EntityVariant,
    pub clip:Clip
}

impl Entity {
    pub fn cell(&self) -> (i32, i32) {
        let cell = self.pos.as_ivec2();
        cell.into()
    }
    pub fn body(&self) -> Body<'_> {
        Body::Entity(self)
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            id: Default::default(),
            pos: Default::default(),
            is_player: Default::default(),
            is_goal: Default::default(),
            update: |_, _, _| {},
            vel: Default::default(),
            is_touching_floor: false,
            delete_me: false,
            timer0: Default::default(),
            dir_x: Default::default(),
            variant:EntityVariant::Unknown,
            pos_start: Default::default(),
            clip:Clip::Clip
        }
    }
}

#[derive(Debug)]
#[derive(Default)]
pub enum DirX {
    Left,
    #[default]
    Right,
}


#[derive(Debug)]
pub enum EntityVariant {
    Unknown,
    Player {
        skin:u32
    },
    Goal,
    Coin,
    Cloud
}

#[derive(Debug)]
pub enum Clip {
    Clip,
    NoClip
}