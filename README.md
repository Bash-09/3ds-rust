
# Getting Started
- Follow the steps on the [Getting started](https://github.com/rust3ds/ctru-rs/wiki/Getting-Started) page of the rust3ds org on Github to setup your dev environment. 
- Run `git submodule update --init` to include the `citro3d-rs` submodule required to build the 3DS app
- Inside the `app` folder, run `rustup override set nightly` to tell Cargo to use Rust nightly when compiling that project
- Use `cargo 3ds build --release` or `cargo 3ds run --address <3dslink address> --release` inside `app` to test the 3DS app (or alternatively run the examples in the `citro3d-rs/citro3d` repo/folder) (be sure to use release mode, as debug will be very slow to deserialise and load the model data at startup, resulting in a black screen for a long time before it actually renders)

After building, the output files will be in `target/armv6k-nintendo-3ds/release`. `app.elf` can be run directly in an emulator like Citra.

# Repository structure
- `app` - The main 3DS app I'm working on
- `core3d` - A crate for handling much of the 3D data and processing in the 3DS app
- `preprocessor` - A desktop app that parses a `gltf`/`glb` file and uses `core3d` to structure and serialize the data so it can be easily imported and used in the main 3DS app
- `citro3d-rs` - My fork of the Rust wrapper for the 3DS GPU driver. This is included as a submodule so it can be kept in a separate repo but still easily have the 3DS project refer to a local and easily-modifiable copy of the crate

# Exporting from Blender and Preprocessing
The app can currently import and render an animated model exported from Blender in the GLTF format.

The Blender model must have a maximum of 18 bones, as that is the limit I've set in the shader (physical limit of the GPU with the current implementation is ~20 bones).

When exporting a model as gltf/glb, change `Skinning -> Bone Influence` to `3`, and make sure to enable `Mesh -> Apply Modifiers` if you have any modifiers that haven't already been applied to the model, such as `Decimate`. Animations and Textures should be included in the export, I left the `Materials` settings as `Export` and `Automatic`, and `Animation` enabled and with default settings.

To use the preprocessor, run it inside the `preprocessor` folder with `cargo run --release -- <input_file_path.glb> <output_file_path>`. This will output a file which is a binary serialisation of the `Model` struct from `core3d`. This can then be included in the 3DS app with `const MODEL_BYTES: &[u8] = include_bytes!("<file_path>");` or using `romfs`, and deserialised with `let model: Model = rmp_serde::from_slice(MODEL_BYTES).unwrap()`.

# Other
Feel free to have a look at `3DS programming experience.pdf` to see a little presentation I made for my friends about part of my experience with getting all this set up.
