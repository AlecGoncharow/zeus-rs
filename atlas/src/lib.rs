pub mod camera;
pub mod entity;
pub mod message;
pub mod proc_gen;
pub mod rendering;
pub mod vertex;

pub use rand;

pub use pantheon::Color;

pub mod prelude {
    use pantheon::graphics::prelude::*;
    pub struct Handles<'a> {
        pub camera_uniforms: BufferHandle<'a>,
        pub reflected_camera_uniforms: BufferHandle<'a>,
        pub depth_texture: TextureHandle<'a>,
        pub reflection_texture: TextureHandle<'a>,
        pub refraction_texture: TextureHandle<'a>,
        pub shaded_pass: PassHandle<'a>,

        pub global_light_uniform: BufferHandle<'a>,
        pub global_shadow_uniforms: BufferHandle<'a>,
        pub global_shadow_bake_uniforms: [BufferHandle<'a>; 4],
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
