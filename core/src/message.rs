use hermes::message::Messageable;
#[derive(Clone, Copy, Debug)]
pub enum GameMessage {
    GetId,
    SyncWorld,
    Ping,
    Interact,
    MovePlayer,
    Player,
}

impl Messageable for GameMessage {}

#[derive(Clone, Copy, Debug)]
pub struct F2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Complex {
    pub a: u32,
    pub b: bool,
    pub c: f32,
    pub d: [F2; 2],
}
