
pub enum Event {
    PickupCoin,
    Won,
    Died,
    PickupExtraLife,
    PlayerJump,
    GameOver {
        score:u32
    }
}