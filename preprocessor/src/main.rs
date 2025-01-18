pub mod model;
pub mod texture;

pub fn main() {
    let args: Vec<String> = std::env::args().collect();

    let src_path = args.get(1).expect("Needs a source file");

    let _ = model::load_gltf(src_path);
}
