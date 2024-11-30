use core3d::Vertex;
use glam::{Vec2, Vec3};

pub static QUAD_VERTS: &[Vertex] = &[
    // Top Right
    Vertex {
        pos: Vec3::new(0.5, 0.5, -3.0),
        norm: Vec3::new(0.0, 0.0, -1.0),
        tc: Vec2::new(1.0, 1.0),
        joints: [0, 0, 0],
        weights: Vec3::new(1.0, 0.0, 0.0),
    },
    // Top Left
    Vertex {
        pos: Vec3::new(-0.5, 0.5, -3.0),
        norm: Vec3::new(0.0, 0.0, -1.0),
        tc: Vec2::new(0.0, 1.0),
        joints: [0, 0, 0],
        weights: Vec3::new(1.0, 0.0, 0.0),
    },
    // Bottom Left
    Vertex {
        pos: Vec3::new(-0.5, -0.5, -3.0),
        norm: Vec3::new(0.0, 0.0, -1.0),
        tc: Vec2::new(0.0, 0.0),
        joints: [0, 0, 0],
        weights: Vec3::new(1.0, 0.0, 0.0),
    },
    // Bottom Right
    Vertex {
        pos: Vec3::new(0.5, -0.5, -3.0),
        norm: Vec3::new(0.0, 0.0, -1.0),
        tc: Vec2::new(1.0, 0.0),
        joints: [0, 0, 0],
        weights: Vec3::new(1.0, 0.0, 0.0),
    },
];

pub static QUAD_INDS: &[u8] = &[0, 1, 2, 0, 2, 3];
