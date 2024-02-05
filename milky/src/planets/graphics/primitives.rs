#[rustfmt::skip]
pub fn quad_vertices() -> &'static [f32] {
    &[
        -0.5,  0.5, 0.0, 0.0,
        -0.5, -0.5, 0.0, 1.0,
         0.5, -0.5, 1.0, 1.0,
         0.5,  0.5, 1.0, 0.0,
    ]
}

pub fn quad_indices() -> &'static [u32] {
    &[0, 1, 2, 0, 2, 3]
}
