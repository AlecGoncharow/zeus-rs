use criterion::{criterion_group, criterion_main, Criterion};
use engine::math::Mat4;
use engine::math::Vec4;

fn invert(c: &mut Criterion) {
    let mat: Mat4 = (
        (4.0, 0.0, 0.0, 1.0),
        (0.0, 0.0, 1.0, 0.0),
        (0.0, 2.0, 2.0, 0.0),
        (0.0, 0.0, 0.0, 1.0),
    )
        .into();

    c.bench_function("mat4 invert", |b| b.iter(|| mat.invert()));

    let invert = mat.invert().unwrap();
    c.bench_function("mat4 invert invert", |b| b.iter(|| invert.invert()));

    let mat = Mat4 {
        x: Vec4 {
            x: -0.9781477,
            y: 0.00821474,
            z: 0.20774916,
            w: 0.0,
        },
        y: Vec4 {
            x: 0.0,
            y: -0.9992192,
            z: 0.03951076,
            w: 0.0,
        },
        z: Vec4 {
            x: -0.20791149,
            y: -0.038647354,
            z: -0.9773839,
            w: 0.0,
        },
        w: Vec4 {
            x: 0.47994256,
            y: 1.5245106,
            z: -14.167097,
            w: 1.0,
        },
    };

    c.bench_function("mat4 from camera invert", |b| b.iter(|| mat.invert()));

    let invert = mat.invert().unwrap();
    let ident = Mat4::scalar_from_one(1);
    c.bench_function("mat4 from camera invert invert", |b| {
        b.iter(|| invert.invert())
    });

    c.bench_function(
        "mat4 from camera invert and multiply with self for sanity check",
        |b| {
            b.iter(|| {
                let invert = mat.invert();
                let invert_mat = invert.unwrap() * mat;
                if !(invert_mat == ident) {
                    eprintln!("expected: {:#?}, got: {:#?}", ident, invert_mat);
                }
            })
        },
    );
}

criterion_group!(benches, invert);
criterion_main!(benches);
