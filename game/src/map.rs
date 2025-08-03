
pub trait Map {
    fn background(&self) -> (u8, u8, u8);
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn tile(&self, x: i32, y: i32) -> MapTile;
}

#[derive(Default)]
pub struct MapTile {
    pub is_player: bool,
    pub is_goal: bool,
    pub is_block: bool,
    pub is_cloud:bool,
    pub is_foreground: bool,
    pub is_entity: bool,
    pub is_coin:bool,
    pub is_deadly:bool,
    pub variant: u32,
}
