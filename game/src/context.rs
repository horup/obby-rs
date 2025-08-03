use std::rc::Rc;
use glam::Vec2;
use crate::Map;

pub trait Context {
    fn map(&mut self, name: &str) -> MapResult;
    fn dt(&self) -> f32;
    fn d_pad(&self) -> Vec2;
    fn is_key_down(&self, key: Keys) -> bool;
    fn is_key_pressed(&self, key:Keys) -> bool;
    fn is_any_key_pressed(&self) -> bool;
    fn map_list(&self) -> &Vec<String>;
    fn rand_f32(&self) -> f32;
}

pub enum MapResult {
    NotFound,
    Pending,
    Ok(Rc<dyn Map>),
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Keys {
    Space,
    Left,
    Right
}
