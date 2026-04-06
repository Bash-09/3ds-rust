use citro3d::{
    attrib,
    macros::include_shader,
    math::{self, ClipPlanes, Projection},
};

pub const VERTEX_SHADER: &[u8] = include_shader!("../shaders/vshader.pica");

pub fn attr_info() -> attrib::Info {
    use attrib::Register;
    // Configure attributes for use with the vertex shader
    let mut attr_info = attrib::Info::new();

    // Pos
    attr_info
        .add_loader(Register::V0, attrib::Format::Float, 3)
        .unwrap();

    // Norm
    attr_info
        .add_loader(Register::V1, attrib::Format::Float, 3)
        .unwrap();

    // TC
    attr_info
        .add_loader(Register::V2, attrib::Format::Float, 2)
        .unwrap();

    // Joints
    attr_info
        .add_loader(Register::V3, attrib::Format::UnsignedByte, 3)
        .unwrap();

    // Weights
    attr_info
        .add_loader(Register::V4, attrib::Format::Float, 3)
        .unwrap();

    attr_info
}

pub fn screen_proj() -> math::Matrix4 {
    let vertical_fov = 40.0_f32.to_radians();
    let clip_planes = ClipPlanes {
        near: 0.1,
        far: 10.0,
    };

    // Mat4::perspective_lh(
    //     vertical_fov,
    //     citro3d::math::AspectRatio::TopScreen.into(),
    //     0.1,
    //     10.0,
    // )

    Projection::perspective(
        vertical_fov,
        citro3d::math::AspectRatio::TopScreen,
        clip_planes,
    )
    .into()
}
