use std::{collections::HashMap, rc::Rc};
use endlessgrid::Grid;
use glam::Vec2;
use crate::{spawn, Body, Context, Entity, Event, Map, MapResult, Tile};

#[derive(Default)]
pub struct Game {
    pub score:u32,
    pub level_current: u32,
    pub map_next: String,
    pub map_current: Option<Rc<dyn Map>>,
    pub grid: Grid<Tile>,
    pub grid_width: u32,
    pub grid_height: u32,
    pub entities: HashMap<u32, Entity>,
    pub center_text: String,
    pub player:u32,
    pub events:Vec<Event>,
    pub pause:bool,
    pub elapsed_total_sec:f32,
    pub lives_extra:i32,
    pub coins:u32,
    pub skin_chosen:u32,
    pub next_id:u32
}

impl Game {
    pub fn init(&mut self, ctx: &mut dyn Context) {
        let _ = ctx;
        self.lives_extra = 3;
        self.map_next = ctx.map_list().first().expect("").clone();
    }

    pub fn next_level(&mut self, ctx: &mut dyn Context) {
        let map_list = ctx.map_list();
        let next_map = self.level_current + 1;
        if let Some(map) = map_list.get(next_map as usize) {
            self.level_current = next_map;
            self.map_next = map.clone();
        } else {
            self.events.push(Event::GameOver { score: self.score });
        }
    }

    pub fn bodies(&'_ self, index: (i32, i32)) -> Vec<Body<'_>> {
        let mut v = Vec::default();
        let a = 1;
        for dy in -a..=a {
            for dx in -a..=a {
                let index = (index.0 + dx, index.1 + dy);
               
                if let Some(cell) = self.grid.get(index) {
                    if cell.is_block {
                        v.push(Body::Block(index, cell));
                    }
                } else if index.0 < 0 || index.0 >= self.grid_width as i32 {
                    v.push(Body::Void(index));
                }
            }
        }

        for e in self.entities.values() {
            match e.clip {
                crate::Clip::Clip => {
                    let body = Body::Entity(e);
                    v.push(body);
                },
                crate::Clip::NoClip => {},
            }
        }
        v
    }

    pub fn restart(&mut self, ctx:&mut dyn Context, whole_game:bool) {
        if whole_game {
            *self = Game {
                lives_extra:3,
                map_next:ctx.map_list().first().expect("").clone(),
                skin_chosen:self.skin_chosen,
                ..Default::default()
            };
        } else {
            *self = Game {
                level_current:self.level_current,
                map_current:self.map_current.take(),
                score:self.score,
                elapsed_total_sec:self.elapsed_total_sec,
                coins:self.coins,
                lives_extra:self.lives_extra,
                skin_chosen:self.skin_chosen,
                ..Default::default()
            };
        }
        let _ = ctx;
       
        if let Some(map) = self.map_current.clone() {
            for y in 0..map.height() {
                for x in 0..map.width() {
                    let tile = map.tile(x as i32, y as i32);
                    if !tile.is_entity {
                        self.grid.insert(
                            (x as i32, y as i32),
                            Tile {
                                is_block: tile.is_block,
                                variant: tile.variant,
                                is_foreground: tile.is_foreground,
                                is_deadly: tile.is_deadly,
                            },
                        );
                    }

                    if tile.is_player {
                        spawn::spawn_player(self, Vec2::new(x as f32 + 0.5, y as f32 + 0.5));
                    }

                    if tile.is_goal {
                        spawn::spawn_goal(self, Vec2::new(x as f32 + 0.5, y as f32 + 0.5));
                    }

                    if tile.is_coin {
                        spawn::spawn_coin(self, Vec2::new(x as f32 + 0.5, y as f32 + 0.5));
                    }

                    if tile.is_cloud {
                        spawn::spawn_cloud(self, Vec2::new(x as f32 + 0.5, y as f32 + 0.5));
                    }
                }
            }
            self.grid_width = map.width();
            self.grid_height = map.height();
            self.map_current = Some(map);
        }
    }

    pub fn update(&mut self, ctx: &mut dyn Context) {
        if !self.map_next.is_empty() {
            match ctx.map(&self.map_next) {
                MapResult::NotFound => {
                    println!("failed to find map with name {}", self.map_next);
                    self.map_next = "".to_string();
                }
                MapResult::Pending => {
                    return;
                }
                MapResult::Ok(map) => {
                    self.map_current = Some(map);
                    self.map_next = "".to_string();
                    self.restart(ctx, false);
                }
            }
        }

        let mut ids: Vec<u32> = self.entities.keys().map(|v| v.to_owned()).collect();
        for id in ids.drain(..) {
            let Some(mut e) = self.entities.remove(&id) else {
                continue;
            };
            (e.update)(&mut e, self, ctx);
            if !e.delete_me {
                self.entities.insert(id, e);
            }
        }
        
        if !self.pause {
            self.elapsed_total_sec += ctx.dt();
        }
    }

    pub fn spawn_entity(&mut self) -> &mut Entity {
        self.next_id += 1;
        let uuid = self.next_id;
        self.entities.insert(
            uuid,
            Entity {
                id: uuid,
                ..Default::default()
            },
        );
        self.entities.get_mut(&uuid).unwrap()
    }
}