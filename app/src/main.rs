#![feature(allocator_api)]

use citro3d::{
    buffer::{self, Buffer, Primitive},
    math,
    render::{ClearFlags, Target},
    shader::{self},
    texenv,
    texture::{self, Face},
};
use core3d::Model;
use ctru::{
    linear::LinearAllocator,
    prelude::*,
    services::gfx::{RawFrameBuffer, Screen},
};
use graphics::{screen_proj, VERTEX_SHADER};

pub mod app;
pub mod graphics;
pub mod quad;
pub mod util;

const CLEAR_COL: u32 = 0x68_B0_D8_FF;

fn main() {
    let gfx = Gfx::new().unwrap();
    let apt = Apt::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let mut citro = citro3d::Instance::new().unwrap();

    let total_linear_heap_size = LinearAllocator::free_space();

    let _romfs = ctru::services::romfs::RomFS::new().unwrap();

    let _console = Console::new(gfx.bottom_screen.borrow_mut());
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

    // let counter: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
    // {
    //     let counter = counter.clone();
    //     util::spawn_thread(1, move || loop {
    //         *counter.lock().expect("Couldn't lock") += 1;
    //     });
    // }

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
    let shader = shader::Library::from_bytes(VERTEX_SHADER).unwrap();
    let program = { shader::Program::new(shader.get(0).unwrap()).unwrap() };

    let start_time = unsafe { ctru_sys::osGetTime() };

    let attr_info = graphics::attr_info();

    println!("Loading assets...");

    // Load exported model
    let model: Model = {
        let model_bytes =
            std::fs::read("romfs:/Bash_3DS.model").expect("Couldn't load model from romfs");
        rmp_serde::from_slice(&model_bytes).expect("Failed to deserialize model")
    };

    let end_time = unsafe { ctru_sys::osGetTime() };
    let setup_time = end_time - start_time;
    println!("Took {setup_time}ms to deserialise model bundle.");

    let mut model1_info = buffer::Info::new();
    model1_info
        .add(Buffer::new(&model.meshes[0].verts), attr_info.permutation())
        .unwrap();
    let mut model1_inds = Vec::new_in(LinearAllocator);
    model1_inds.extend_from_slice(&model.meshes[0].inds);

    let mut model2_info = buffer::Info::new();
    model2_info
        .add(Buffer::new(&model.meshes[1].verts), attr_info.permutation())
        .unwrap();
    let mut model2_inds = Vec::new_in(LinearAllocator);
    model2_inds.extend_from_slice(&model.meshes[1].inds);

    let tex_ind = 0;

    // Create texture
    let mut texture1 = texture::Texture::new(texture::TextureParameters::new_2d(
        model.textures[tex_ind].width,
        model.textures[tex_ind].height,
        texture::ColorFormat::Rgba8,
    ))
    .unwrap();
    let mut tex_bytes = Vec::with_capacity_in(model.textures[tex_ind].data.len(), LinearAllocator);
    tex_bytes.extend_from_slice(&model.textures[tex_ind].data);
    texture1
        .load_image(&tex_bytes, Face::default())
        .expect("Failed to load texture bytes");

    // Create texture
    let mut texture2 = texture::Texture::new(texture::TextureParameters::new_2d(
        model.textures[1].width,
        model.textures[1].height,
        texture::ColorFormat::Rgba8,
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
        texenv::TexEnv::new().src(texenv::Mode::BOTH, texenv::Source::Texture0, None, None);

    let mut total_frame_time: u64 = 0;

    let mut t: f32 = 0.0;

    println!("\x1b[29;16HPress Start to exit");

    while apt.main_loop() {
        t += 0.16;
        let frame_start_time = unsafe { ctru_sys::osGetTime() };

        let mut model_matrix = math::Matrix4::identity();
        model_matrix.rotate_y((4.0 * t).to_radians());
        model_matrix.translate(0.0, -1.0, -4.5);
        let mvp = screen_proj * model_matrix;

        let animated_pose = model.animations[0].sample(t * 0.25);
        let joint_transforms = model.skeleton.apply_pose_to_joints(&animated_pose).unwrap();

        hid.scan_input();

        if hid.keys_down().contains(KeyPad::START) {
            break;
        }

        let used_linear_mem = total_linear_heap_size - LinearAllocator::free_space();
        println!("\x1b[2;0H Frame time: {total_frame_time}ms");

        println!("\x1b[4;0H Memory usage:");
        println!(
            "\x1b[5;0H   Linear: {}kB / {}kB ({:.2}%)",
            used_linear_mem / 1024,
            total_linear_heap_size / 1024,
            (used_linear_mem as f32 / total_linear_heap_size as f32) * 100.0
        );

        // println!(
        //     "\x1b[7;0H Background counter: {}",
        //     counter.lock().expect("Couldn't lock from main thread :()")
        // );

        // Application memory just sits at 100%, I assume because the allocator is claiming it all on initialisation :c
        // println!(
        //     "\x1b[6;0H   Application: {}MB / {}MB ({:.2}%)",
        //     MemRegion::Application.used() / 1048576,
        //     MemRegion::Application.size() / 1048576,
        //     (MemRegion::Application.used() as f32 / MemRegion::Application.size() as f32) * 100.0
        // );

        // Render
        citro.render_frame_with(|mut frame| {
            screen_target.clear(ClearFlags::ALL, CLEAR_COL, 0);

            frame.bind_program(&program);
            frame.set_attr_info(&attr_info);
            frame.select_render_target(&screen_target).unwrap();
            frame.bind_texture(texture::Index::Texture0, &texture1);
            frame.set_texenvs(&[textured_stage]);
            frame.bind_vertex_uniform(uniform_proj, mvp);
            frame.bind_vertex_uniform(uniform_joint, joint_transforms.as_slice());
            frame.draw_elements(Primitive::Triangles, &model1_info, &model1_inds);

            frame.bind_texture(texture::Index::Texture0, &texture2);
            frame.draw_elements(Primitive::Triangles, &model2_info, &model2_inds);

            frame
        });

        total_frame_time = unsafe { ctru_sys::osGetTime() - frame_start_time };
    }
}
