use std::{
    collections::HashMap,
    io::{Cursor, Error},
    rc::Rc,
};

use game::MapResult;
use macroquad::{audio::Sound, file::load_file, math::Rect, texture::Texture2D};

use crate::Map;

pub struct Atlas {
    pub col: u16,
    pub rows: u16,
    pub texture: Texture2D,
}

impl Atlas {
    pub fn new(col: u16, rows: u16, texture: Texture2D) -> Self {
        texture.set_filter(macroquad::texture::FilterMode::Nearest);
        Self { col, rows, texture }
    }

    pub fn index(&self, index: f32) -> Rect {
        let w = self.texture.width() / self.col as f32;
        let h = self.texture.height() / self.rows as f32;
        let idx = index as u16;
        let col = idx % self.col;
        let row = idx / self.col;
        let x = col as f32 * w;
        let y = row as f32 * h;
        Rect { x, y, w, h }
    }
}

pub struct Assets {
    pub maps: HashMap<String, MapResult>,
    pub maps_pending: Vec<String>,
    pub tileset: Atlas,
    pub sfx: HashMap<String, Vec<Sound>>,
}

struct TiledReader {
    pub resources: HashMap<String, Rc<[u8]>>,
}
impl tiled::ResourceReader for TiledReader {
    type Resource = Cursor<Rc<[u8]>>;

    type Error = Error;

    fn read_from(
        &mut self,
        path: &std::path::Path,
    ) -> std::result::Result<Self::Resource, Self::Error> {
        match self.resources.get(path.to_str().unwrap_or_default()) {
            Some(res) => Result::Ok(Cursor::new(res.clone())),
            None => Result::Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file was not found",
            )),
        }
    }
}

impl Assets {
    pub fn load_map(&mut self, path: &str) -> MapResult {
        if !self.maps.contains_key(path) {
            self.maps.insert(path.to_string(), MapResult::Pending);
            self.maps_pending.push(path.to_string());
        }

        let map_result = self.maps.get(path).unwrap();
        match map_result {
            MapResult::NotFound => MapResult::NotFound,
            MapResult::Pending => MapResult::Pending,
            MapResult::Ok(map) => MapResult::Ok(map.clone()),
        }
    }

    pub async fn load_pending(&mut self) {
        // load maps
        for path in self.maps_pending.drain(..) {
            let mut resources = HashMap::new();
            resources.insert(
                "res/maps/tileset.tsx".to_string(),
                load_file("res/maps/tileset.tsx")
                    .await
                    .unwrap()
                    .into_boxed_slice()
                    .into(),
            );
            resources.insert(
                path.clone(),
                load_file(&path).await.unwrap().into_boxed_slice().into(),
            );
            let mut loader = tiled::Loader::with_reader(TiledReader { resources });

            let map = loader.load_tmx_map(&path);
            match map {
                Ok(map) => {
                    let map = Map { tiled_map: map };
                    let res = game::MapResult::Ok(Rc::new(map));
                    self.maps.insert(path.to_string(), res);
                }
                Err(_) => {
                    self.maps.insert(path.to_owned(), game::MapResult::NotFound);
                }
            }
        }
    }
}
