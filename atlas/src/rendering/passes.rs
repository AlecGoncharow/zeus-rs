use bitflags::bitflags;

const LAST_PASS: usize = 8;

bitflags! {
    #[rustfmt::skip]
    pub struct Passes: usize {
        const DIRECTIONAL_SHADOW_0  = 1 << 0;
        const DIRECTIONAL_SHADOW_1  = 1 << 1;
        const DIRECTIONAL_SHADOW_2  = 1 << 2;
        const DIRECTIONAL_SHADOW_3  = 1 << 3;
        const REFLECTION            = 1 << 4;
        const REFRACTION            = 1 << 5;
        const SHADED_RENDER         = 1 << 6;
        const WATER                 = 1 << 7;
        const BASIC_TEXURED         = 1 << LAST_PASS;

        const SHADED_BUNDLE         = 0b0_0111_1111;
    }
}

/* Soon TM
bitflags! {
    #[rustfmt::skip]
    pub struct Passes: usize {
        const REFLECTION            = 1 << 0;
        const REFRACTION            = 1 << 1;
        const SHADED_RENDER         = 1 << 2;
        const WATER                 = 1 << 3;
        const BASIC_TEXURED         = 1 << 4;

        const SHADED_BUNDLE         = 0b0_0111;
    }
}
*/
