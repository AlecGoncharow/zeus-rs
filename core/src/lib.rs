pub mod camera;
pub mod entity;
pub mod message;
pub mod proc_gen;

pub use rand;

pub use pantheon::Color;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
