use std::path::Path;

use core3d::*;
use glam::{Mat4, Quat, Vec2, Vec3};
use gltf::animation::{util::ReadOutputs, Property};

use crate::texture::{swizzle, IDX_A, IDX_B, IDX_G, IDX_R};

const MAX_JOINTS: u16 = 19;

#[allow(clippy::too_many_lines)]
pub fn load_gltf<P: AsRef<Path>>(file: P) -> Model {
    let (gltf, buffers, images) = gltf::import(file).expect("Couldn't import gltf file");

    let mut model = Model {
        meshes: Vec::new(),
        textures: Vec::new(),
        skeleton: Skeleton { joints: Vec::new() },
        animations: Vec::new(),
    };

    // Textures
    for data in images {
        let mut texture = Texture {
            data: vec![0; (data.width * data.height * 4) as usize],
            width: data.width as u16,
            height: data.height as u16,
        };
        println!(
            "Extracting texture {} x {} ({} bytes)",
            texture.width,
            texture.height,
            texture.data.len()
        );

        // Rearrange the texture into the correct layout
        for x in 0..data.width {
            for y in 0..data.height {
                let src_idx = ((data.width - y - 1) * data.width + x) as usize;
                let dst_idx = swizzle(x, y, data.width);

                texture.data[dst_idx * 4 + IDX_R] = data.pixels[src_idx * 3 + 0];
                texture.data[dst_idx * 4 + IDX_G] = data.pixels[src_idx * 3 + 1];
                texture.data[dst_idx * 4 + IDX_B] = data.pixels[src_idx * 3 + 2];
                texture.data[dst_idx * 4 + IDX_A] = 255;
            }
        }

        model.textures.push(texture);
    }

    // Mesh
    let gltf_mesh = gltf.meshes().next().expect("GLTF did not contain a mesh");

    for primitive in gltf_mesh.primitives() {
        println!("Extracting primitive");
        let mut mesh = Mesh {
            verts: Vec::new(),
            inds: Vec::new(),
            texture: 0,
        };

        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        // Material/Texture
        let tex_index = primitive
            .material()
            .pbr_metallic_roughness()
            .base_color_texture()
            .map(|t| t.texture().source().index() as u8);
        if tex_index.is_none() {
            println!("Primitive is missing a texture");
        }
        mesh.texture = tex_index.unwrap_or(0);

        // Positions
        for pos in reader.read_positions().expect("No vertex positions") {
            mesh.verts.push(Vertex {
                pos: Vec3::new(pos[0], pos[1], pos[2]),
                ..Default::default()
            });
        }

        // Normals
        if let Some(norms) = reader.read_normals() {
            for (i, norm) in norms.enumerate() {
                mesh.verts[i].norm = Vec3::new(norm[0], norm[1], norm[2]);
            }
        } else {
            println!("No normals found, defaulting to [0, 0, 0]");
        }

        // Tex coords
        if let Some(tcs) = reader.read_tex_coords(0) {
            for (i, tc) in tcs.into_f32().enumerate() {
                mesh.verts[i].tc = Vec2::new(tc[0], tc[1]);
            }
        } else {
            println!("No tex coords found, defaulting to [0, 0]");
        }

        // Joints
        if let Some(joints) = reader.read_joints(0) {
            for (i, joint_ids) in joints.into_u16().enumerate() {
                assert!(
                    joint_ids[0] < MAX_JOINTS
                        && joint_ids[1] < MAX_JOINTS
                        && joint_ids[2] < MAX_JOINTS
                );
                mesh.verts[i].joints = [joint_ids[0] as u8, joint_ids[1] as u8, joint_ids[2] as u8];
            }
        } else {
            println!("No joints found, defaulting to [0, 0, 0]");
        }

        // Weights
        if let Some(weights) = reader.read_weights(0) {
            for (i, weights) in weights.into_f32().enumerate() {
                mesh.verts[i].weights = Vec3::new(weights[0], weights[1], weights[2]).normalize();
            }
        } else {
            println!("No joint weights found, defaulting to [1, 0, 0]");
        }

        // Indices
        if let Some(indices) = reader.read_indices() {
            for idx in indices.into_u32() {
                mesh.inds.push(
                    idx.try_into()
                        .expect("Index can't fit in u16, probably too many vertices."),
                );
            }
        }

        model.meshes.push(mesh);
    }

    // Skeleton
    if let Some(skin) = gltf.skins().next() {
        println!("Extracting skeleton");

        for joint in skin.joints() {
            let (pos, rot, scale) = joint.transform().decomposed();
            let pos = Vec3::new(pos[0], pos[1], pos[2]);
            let rot = Quat::from_xyzw(rot[0], rot[1], rot[2], rot[3]);
            let scale = Vec3::new(scale[0], scale[1], scale[2]);
            let children = joint
                .children()
                .map(|c| c.index().try_into().expect("Child joint out of bounds"))
                .collect();
            model.skeleton.joints.push(Joint {
                parent: None,
                index: joint.index() as u8,
                name: joint.name().map_or(String::new(), String::from),
                base_transform: JointTransform { pos, rot, scale },
                inverse_bind_matrix: Mat4::IDENTITY,
                children,
            });
        }

        // Inverse bind matrices
        let reader = skin.reader(|buffer| Some(&buffers[buffer.index()]));
        for (i, ibm) in reader
            .read_inverse_bind_matrices()
            .expect("No Inverse bind matrices??")
            .enumerate()
        {
            model.skeleton.joints[i].inverse_bind_matrix = Mat4::from_cols_array_2d(&ibm);
        }
    } else {
        println!("No skeleton found, defaulting to identity skeleton");
        model.skeleton.joints.push(Joint::default());
    }

    // Remap children
    let mut index_map = vec![0; model.skeleton.joints.len()];
    for (i, j) in model.skeleton.joints.iter().enumerate() {
        index_map[j.index as usize] = i as u8;
    }
    for i in model
        .skeleton
        .joints
        .iter_mut()
        .flat_map(|j| &mut j.children)
    {
        *i = index_map[*i as usize];
    }

    // Parents
    let mut parents: Vec<Option<u8>> = vec![None; model.skeleton.joints.len()];
    for (i, j) in model.skeleton.joints.iter().enumerate() {
        for c in &j.children {
            parents[*c as usize] = Some(i as u8);
        }
    }

    for (i, j) in model.skeleton.joints.iter_mut().enumerate() {
        j.parent = parents[i];
    }

    // Animations
    for animation in gltf.animations() {
        let channels = animation.channels();
        let mut animation = Animation {
            name: animation.name().map_or(String::new(), String::from),
            joints: vec![JointAnimation::default(); model.skeleton.joints.len()],
        };

        'channels: for c in channels {
            let bone_index = index_map[c.target().node().index()] as usize;
            let reader = c.reader(|buffer| Some(&buffers[buffer.index()]));

            for i in reader.read_inputs().expect("No inputs?") {
                match c.target().property() {
                    Property::Translation => animation.joints[bone_index]
                        .translations
                        .push((i, Vec3::ZERO)),
                    Property::Rotation => animation.joints[bone_index]
                        .rotations
                        .push((i, Quat::IDENTITY)),
                    Property::Scale => animation.joints[bone_index].scales.push((i, Vec3::ONE)),
                    Property::MorphTargetWeights => {
                        println!("Don't support Morph Targets");
                        continue 'channels;
                    }
                }
            }

            match reader.read_outputs().expect("No outputs?") {
                ReadOutputs::Translations(o) => {
                    for (i, t) in o.enumerate() {
                        animation.joints[bone_index].translations[i].1 = Vec3::from_array(t);
                    }
                }
                ReadOutputs::Rotations(o) => {
                    for (i, t) in o.into_f32().enumerate() {
                        animation.joints[bone_index].rotations[i].1 = Quat::from_array(t);
                    }
                }
                ReadOutputs::Scales(o) => {
                    for (i, t) in o.enumerate() {
                        animation.joints[bone_index].scales[i].1 = Vec3::from_array(t);
                    }
                }
                ReadOutputs::MorphTargetWeights(_) => {
                    println!("Don't support morph targets, but also wtf this shouldn't have made it here.");
                    continue 'channels;
                }
            }
        }

        model.animations.push(animation);
    }

    model
}
