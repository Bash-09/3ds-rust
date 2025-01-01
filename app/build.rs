fn main() {
    println!("cargo::rerun-if-changed=assets/*");

    // Preprocess anything in the assets folder and forward it to the romfs folder
    let files = std::fs::read_dir("assets")
        .expect("Couldn't read assets folder")
        .into_iter()
        .map(|r| r.expect("Error while reading assets folder files"));

    for f in files {
        let name = f.file_name();
        let name = name.to_string_lossy().to_owned();
        let (raw_name, extension) = {
            let mut a = name.split('.');
            (a.next().unwrap(), a.last().unwrap_or(""))
        };

        let file_contents =
            std::fs::read(f.path()).expect("Failed to read file \"{name}\" from assets folder.");

        let (out, new_ext) = match extension {
            "gltf" | "glb" => {
                let bundle = preprocessor::load_gltf(f.path());
                let bytes = rmp_serde::to_vec(&bundle).unwrap();
                (bytes, "model")
            }
            // Pass through any other files directly
            _ => (file_contents, extension),
        };

        let new_name = if extension.is_empty() {
            name.to_string()
        } else {
            format!("{raw_name}.{new_ext}")
        };
        std::fs::write(format!("romfs/{new_name}"), out)
            .expect("Failed to write \"{new_name}\" into romfs")
    }
}
