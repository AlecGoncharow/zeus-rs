use super::Context;

pub trait AsComponent: AsDrawable + AsMouseable {}

pub trait DrawComponent {
    fn draw(&mut self, ctx: &mut Context);
}

pub trait AsDrawable {
    fn as_drawable(&mut self) -> Option<&mut dyn DrawComponent> {
        None
    }
}

/// this is useful because it allows 3D picking to ignore entities which aren't part of the
/// clickable environment
pub trait MouseComponent {
    // TODO think about x/y/z and hover events
    fn click_start(&mut self, ctx: &mut Context);
    fn click_end(&mut self, ctx: &mut Context);
}

pub trait AsMouseable {
    fn as_mouseable(&mut self) -> Option<&mut dyn MouseComponent> {
        None
    }
}
