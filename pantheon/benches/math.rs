use criterion::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use pantheon::math::prelude::*;
use pantheon::math::Mat4;

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

    /*
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
    */
}

fn vec4(c: &mut Criterion) {
    let v1 = black_box(Vec4::new(24, 64, -23, 1));
    let v2 = black_box(Vec4::new(1000.12312, 235.123213, -0.213451235, 1.0));
    let unit = black_box(Vec4::new(1, 0, 0, 0));
    let scalar = black_box(32.);

    c.bench_function("vec4 dot products", |b| b.iter(|| v1.dot(&v2)));
    c.bench_function("vec4 dot products", |b| b.iter(|| v2.dot(&v1)));

    c.bench_function("vec4 normalize non normal", |b| b.iter(|| v2.unit_vector()));
    c.bench_function("vec4 normalize unit", |b| b.iter(|| unit.unit_vector()));

    c.bench_function("vec4 multiply vecs", |b| b.iter(|| v1.make_comp_mul(&v2)));
    c.bench_function("vec4 multiply vec scalar", |b| b.iter(|| scalar * v1));
    c.bench_function("vec4 multiply vec with vec from scalar", |b| {
        b.iter(|| v1.make_comp_mul(&Vec4::new_from_one(scalar)))
    });
}
fn vec3(c: &mut Criterion) {
    let v1 = black_box(Vec3::new(24, 64, -23));
    let v2 = black_box(Vec3::new(1000.12312, 235.123213, -0.213451235));
    let unit = black_box(Vec3::new(1, 0, 0));
    let scalar = black_box(32.);

    c.bench_function("vec3 dot products", |b| b.iter(|| v1.dot(&v2)));
    c.bench_function("vec3 dot products", |b| b.iter(|| v2.dot(&v1)));

    c.bench_function("vec3 normalize non normal", |b| b.iter(|| v2.unit_vector()));
    c.bench_function("vec3 normalize unit", |b| b.iter(|| unit.unit_vector()));

    c.bench_function("vec3 multiply vecs", |b| b.iter(|| v1.make_comp_mul(&v2)));
    c.bench_function("vec3 multiply vec scalar", |b| b.iter(|| scalar * v1));
    c.bench_function("vec3 multiply vec with vec from scalar", |b| {
        b.iter(|| v1.make_comp_mul(&Vec3::new_from_one(scalar)))
    });
}

fn vec2(c: &mut Criterion) {
    let v1 = black_box(Vec2::new(24, 64));
    let v2 = black_box(Vec2::new(1000.12312, 235.123213));
    let scalar = black_box(22.);

    c.bench_function("vec2 dot products", |b| b.iter(|| v1.dot(&v2)));
    c.bench_function("vec2 dot products", |b| b.iter(|| v2.dot(&v1)));

    c.bench_function("vec2 multiply vecs", |b| b.iter(|| v1.make_comp_mul(&v2)));
    c.bench_function("vec2 multiply vec scalar", |b| b.iter(|| scalar * v1));
    c.bench_function("vec2 multiply vec with vec from scalar", |b| {
        b.iter(|| v1.make_comp_mul(&Vec2::new(scalar, scalar)))
    });
}

fn camera(c: &mut Criterion) {
    let mut u = black_box(Vec3::new(0, 0, 1));
    let mut v = black_box(Vec3::new(-0.6, 0.8, 0.0));
    let mut w = black_box(Vec3::new(0.8, 0.6, 0.));
    let origin = black_box(Vec3::new(0, 20, 20));
    let mut translation = Mat4::identity();
    let mut view = Mat4::identity();

    c.bench_function("camera update view matrix", |b| {
        b.iter(|| {
            (u, v, w) = black_box((u, v, w));
            let uvw = Mat3::new(u, v, w);
            let rotation = uvw.transpose().mat4();

            /*
            self.transform = uvw.mat4();
            self.transform.w = self.origin.vec4_with(1.0);
            */

            let negative_from = -1.0 * black_box(origin);
            translation = Mat4::translation::<f32>(negative_from.into());

            view = rotation * translation;

            black_box(view)
        })
    });
}

criterion_group!(benches, invert, vec4, vec3, vec2, camera);

criterion_main!(benches);
