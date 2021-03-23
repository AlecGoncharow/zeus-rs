use hermes::message::MessageKind;

#[derive(Clone, Copy, Debug)]
pub enum CustomMsg {
    Ping,
    Interact(usize),
    MovePlayer(usize),
    Player(usize),
}

impl MessageKind for CustomMsg {}

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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
