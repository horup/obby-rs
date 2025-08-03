use glam::Vec2;

use crate::{Tile, Entity};

pub enum Body<'a> {
    Entity(&'a Entity),
    Block((i32, i32), &'a Tile),
    Void((i32, i32))
}
impl<'a> cliplib::Body for Body<'a> {
    fn center(&self) -> Vec2 {
        match self {
            Body::Entity(entity) => entity.pos,
            Body::Block(i, _) => Vec2::new(i.0 as f32 + 0.5, i.1 as f32 + 0.5),
            Body::Void(i) => Vec2::new(i.0 as f32 + 0.5, i.1 as f32 + 0.5)
        }
    }

    fn half_extent(&self) -> f32 {
        match self {
            Body::Entity(_) => 0.45,
            Body::Block(_, _) => 0.5,
            Body::Void(_) => 0.5,
        }
    }
}

