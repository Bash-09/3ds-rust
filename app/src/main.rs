#![feature(allocator_api)]

use citro3d::{
    buffer::{self},
    math,
    render::{ClearFlags, Target},
    shader::{self},
    texenv,
    texture::{self, Face},
    RenderPass,
};
use core3d::Model;
use ctru::{
    linear::LinearAllocator,
    prelude::*,
    services::gfx::{RawFrameBuffer, Screen},
};
use graphics::{screen_proj, VERTEX_SHADER};
use quad::{QUAD_INDS, QUAD_VERTS};

pub mod app;
pub mod graphics;
pub mod quad;

const CLEAR_COL: u32 = 0x68_B0_D8_FF;

fn main() {
    let gfx = Gfx::new().unwrap();
    let apt = Apt::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let mut citro = citro3d::Instance::new().unwrap();

    let _console = Console::new(gfx.bottom_screen.borrow_mut());
    let _romfs = ctru::services::romfs::RomFS::new().unwrap();

    std::panic::set_hook(Box::new(|info| {
        println!("Panic: {info}");

        unsafe {
            loop {
                ctru_sys::hidScanInput();
                let keys = ctru_sys::hidKeysDown();
                if KeyPad::from_bits_truncate(keys).contains(KeyPad::START) {
                    break;
                }
            }
        }
    }));

    // Screens and render target
    let mut top_screen = gfx.top_screen.borrow_mut();
    let RawFrameBuffer { width, height, .. } = top_screen.raw_framebuffer();
    let mut screen_target = citro
        .render_target(
            width,
            height,
            top_screen,
            Some(citro3d::render::DepthFormat::Depth24),
        )
        .unwrap();

    // Shader setup
    let program = {
        let shader = shader::Library::from_bytes(VERTEX_SHADER).unwrap();
        shader::Program::new(shader, 0).unwrap()
    };

    let attr_info = graphics::attr_info();

    // Load quad
    // let mut quad_vertices = Vec::with_capacity_in(QUAD_VERTS.len(), LinearAllocator);
    // quad_vertices.extend_from_slice(QUAD_VERTS);

    // let mut quad_buf_info = buffer::Info::new();
    // let quad_slice = quad_buf_info.add(&quad_vertices, &attr_info).unwrap();
    // let quad_inds = quad_slice.index_buffer(QUAD_INDS).unwrap();

    let start_time = unsafe { ctru_sys::osGetTime() };

    // Load exported model
    let model: Model = {
        let model_bytes =
            std::fs::read("romfs:/Bash_3DS.model").expect("Couldn't load model from romfs");
        rmp_serde::from_slice(&model_bytes).expect("Failed to deserialize model")
    };

    let end_time = unsafe { ctru_sys::osGetTime() };
    let setup_time = end_time - start_time;
    println!("Took {setup_time}ms to deserialise model bundle.");

    let mut model1_verts = Vec::with_capacity_in(model.meshes[0].verts.len(), LinearAllocator);
    model1_verts.extend_from_slice(&model.meshes[0].verts);

    let mut model1_buf_info = buffer::Info::new();
    let model1_slice = model1_buf_info.add(&model1_verts, &attr_info).unwrap();
    let model1_inds = model1_slice.index_buffer(&model.meshes[0].inds).unwrap();

    // Create texture
    let mut texture1 = texture::Texture::new(texture::TextureParameters::new_2d(
        model.textures[0].width,
        model.textures[0].height,
        texture::Format::RGBA8,
    ))
    .unwrap();

    let mut tex_bytes = Vec::with_capacity_in(model.textures[0].data.len(), LinearAllocator);
    tex_bytes.extend_from_slice(&model.textures[0].data);

    texture1
        .load_image(&tex_bytes, Face::default())
        .expect("Failed to load texture bytes");

    let mut model2_verts = Vec::with_capacity_in(model.meshes[1].verts.len(), LinearAllocator);
    model2_verts.extend_from_slice(&model.meshes[1].verts);

    let mut model2_buf_info = buffer::Info::new();
    let model2_slice = model2_buf_info.add(&model2_verts, &attr_info).unwrap();
    let model2_inds = model2_slice.index_buffer(&model.meshes[1].inds).unwrap();

    // Create texture
    let mut texture2 = texture::Texture::new(texture::TextureParameters::new_2d(
        model.textures[1].width,
        model.textures[1].height,
        texture::Format::RGBA8,
    ))
    .unwrap();

    let mut tex_bytes = Vec::with_capacity_in(model.textures[1].data.len(), LinearAllocator);
    tex_bytes.extend_from_slice(&model.textures[1].data);

    texture2
        .load_image(&tex_bytes, Face::default())
        .expect("Failed to load texture bytes");

    // Projection and uniform
    let screen_proj = screen_proj();

    let uniform_proj = program.get_uniform("projection").unwrap();
    let uniform_joint = program.get_uniform("jointTransforms").unwrap();

    let textured_stage =
        texenv::TexEnv::new().sources(texenv::Mode::BOTH, texenv::Source::Texture0, None, None);

    let mut total_frame_time: u64 = 0;

    let mut t: f32 = 0.0;

    println!("Hello, World!");
    println!("\x1b[29;16HPress Start to exit");

    while apt.main_loop() {
        t += 0.16;
        let frame_start_time = unsafe { ctru_sys::osGetTime() };

        let mut model_matrix = math::Matrix4::identity();
        model_matrix.rotate_y((4.0 * t).to_radians());
        model_matrix.translate(0.0, -1.0, -4.0);
        let mvp = screen_proj * model_matrix;

        let animated_pose = model.animations[0].sample(t * 0.25);
        let joint_transforms = model.skeleton.apply_pose_to_joints(&animated_pose).unwrap();

        hid.scan_input();

        if hid.keys_down().contains(KeyPad::START) {
            break;
        }

        println!("\x1b[3;0H Frame time: {total_frame_time}ms");

        // Render
        citro.render_frame_with(|frame| {
            screen_target.clear(ClearFlags::ALL, CLEAR_COL, 0);

            let body_pass = RenderPass::new(&program, &screen_target, model1_slice, &attr_info)
                .with_texenv_stages([&textured_stage])
                .with_indices(&model1_inds)
                .with_texture(texture::TexUnit::TexUnit0, &texture1)
                .with_vertex_uniforms([
                    (uniform_proj, mvp.into()),
                    (uniform_joint, joint_transforms.as_slice().into()),
                ]);
            frame.draw(&body_pass).unwrap();

            let wings_pass = body_pass
                .with_vbo(model2_slice, &attr_info)
                .with_indices(&model2_inds)
                .with_texture(texture::TexUnit::TexUnit0, &texture2);
            frame.draw(&wings_pass).unwrap();
        });

        total_frame_time = unsafe { ctru_sys::osGetTime() - frame_start_time };
    }
}
