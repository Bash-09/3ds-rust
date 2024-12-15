use glam::{Mat4, Quat, Vec2, Vec3};
use rkyv::{Archive, Deserialize, Serialize};

pub use glam;
pub use rkyv;

pub mod ik;

#[repr(C)]
#[derive(Clone, Copy, Archive, Serialize, Deserialize, Debug)]
pub struct Vertex {
    pub pos: Vec3,
    pub norm: Vec3,
    pub tc: Vec2,
    /// Indices for which joints influence this vertex
    pub joints: [u8; 3],
    /// How much each of the 3 joints influence this vertex
    pub weights: Vec3,
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
            norm: Vec3::ZERO,
            tc: Vec2::ZERO,
            joints: [0; 3],
            weights: Vec3::new(1.0, 0.0, 0.0),
        }
    }
}

#[derive(Clone, Archive, Serialize, Deserialize, Debug)]
pub struct Mesh {
    pub verts: Vec<Vertex>,
    pub inds: Vec<u16>,
    pub texture: u8,
}

#[derive(Clone, Copy, Archive, Serialize, Deserialize, Debug)]
pub struct JointTransform {
    pub pos: Vec3,
    pub rot: Quat,
    pub scale: Vec3,
}

impl JointTransform {
    #[must_use]
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rot, self.pos)
    }
}

impl From<JointTransform> for Mat4 {
    fn from(val: JointTransform) -> Self {
        val.matrix()
    }
}

impl From<Mat4> for JointTransform {
    fn from(value: Mat4) -> Self {
        let (scale, rot, trans) = value.to_scale_rotation_translation();
        Self {
            pos: trans,
            rot,
            scale,
        }
    }
}

impl Default for JointTransform {
    fn default() -> Self {
        JointTransform {
            pos: Vec3::ZERO,
            rot: Quat::IDENTITY,
            scale: Vec3::ZERO,
        }
    }
}

#[derive(Clone, Archive, Serialize, Deserialize, Debug)]
pub struct Joint {
    pub name: String,
    /// In the joint's local space, relative to its parent
    pub base_transform: JointTransform,
    pub inverse_bind_matrix: Mat4,
    /// Indices into a vector of Joints
    pub children: Vec<u16>,
    pub parent: Option<u8>,
}

impl Default for Joint {
    fn default() -> Self {
        Self {
            name: String::new(),
            base_transform: JointTransform {
                pos: Vec3::ZERO,
                rot: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            inverse_bind_matrix: Mat4::IDENTITY,
            children: Vec::new(),
            parent: None,
        }
    }
}

/// A collection of up to 255 joints making up a skeleton
#[derive(Clone, Archive, Serialize, Deserialize, Debug)]
pub struct Skeleton {
    /// The first joint is the root joint
    pub joints: Vec<Joint>,
}

impl Skeleton {
    #[must_use]
    pub fn pose(&self) -> Vec<JointTransform> {
        self.joints.iter().map(|j| j.base_transform).collect()
    }

    /// Returns the index of the joint with the given name
    #[must_use]
    pub fn find_joint(&self, joint_name: &str) -> Option<usize> {
        self.joints
            .iter()
            .enumerate()
            .find(|(_, j)| j.name == joint_name)
            .map(|(i, _)| i)
    }
}

#[derive(Clone, Archive, Serialize, Deserialize, Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub textures: Vec<Texture>,
    pub skeleton: Skeleton,
    pub animations: Vec<Animation>,
}

#[derive(Clone, Archive, Serialize, Deserialize, Debug)]
pub struct Texture {
    pub data: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Archive, Serialize, Deserialize, Debug, Default)]
pub struct JointAnimation {
    pub translations: Vec<(f32, Vec3)>,
    pub rotations: Vec<(f32, Quat)>,
    pub scales: Vec<(f32, Vec3)>,
}

impl JointAnimation {
    #[must_use]
    pub fn sample(&self, t: f32) -> JointTransform {
        let mut out = JointTransform::default();

        // Translation
        let t_tran = t % self.translations.last().map_or(1.0, |(len, _)| *len);
        let (t_before, tran_before) = self
            .translations
            .iter()
            .find(|(t, _)| *t <= t_tran)
            .copied()
            .unwrap_or((0.0, Vec3::ZERO));
        let (t_after, tran_after) = self
            .translations
            .iter()
            .find(|(t, _)| *t > t_tran)
            .copied()
            .unwrap_or((1.0, Vec3::ZERO));

        let t_interp = (t_tran - t_before) / (t_after - t_before);
        out.pos = tran_before.lerp(tran_after, t_interp);

        // Rotation
        let t_rot = t % self.rotations.last().map_or(1.0, |(len, _)| *len);
        let (t_before, rot_before) = self
            .rotations
            .iter()
            .find(|(t, _)| *t <= t_rot)
            .copied()
            .unwrap_or((0.0, Quat::IDENTITY));
        let (t_after, rot_after) = self
            .rotations
            .iter()
            .find(|(t, _)| *t > t_rot)
            .copied()
            .unwrap_or((1.0, Quat::IDENTITY));

        let t_interp = (t_rot - t_before) / (t_after - t_before);
        out.rot = rot_before.lerp(rot_after, t_interp);

        // Translation
        let t_scale = t % self.scales.last().map_or(1.0, |(len, _)| *len);
        let (t_before, scale_before) = self
            .scales
            .iter()
            .find(|(t, _)| *t <= t_scale)
            .copied()
            .unwrap_or((0.0, Vec3::ZERO));
        let (t_after, scale_after) = self
            .scales
            .iter()
            .find(|(t, _)| *t > t_scale)
            .copied()
            .unwrap_or((1.0, Vec3::ZERO));

        let t_interp = (t_scale - t_before) / (t_after - t_before);
        out.scale = scale_before.lerp(scale_after, t_interp);

        out
    }
}

#[derive(Clone, Archive, Serialize, Deserialize, Debug)]
pub struct Animation {
    pub name: String,
    pub joints: Vec<JointAnimation>,
}

impl Animation {
    #[must_use]
    pub fn sample(&self, t: f32) -> Vec<JointTransform> {
        self.joints
            .iter()
            .map(|joint_anim| joint_anim.sample(t))
            .collect()
    }
}

impl Skeleton {
    /// Calculates the transformation matrices for a set of joint transforms.
    /// Will return None if the number of joint transforms provided does not match
    /// the number of joints in the skeleton.
    #[must_use]
    pub fn apply_pose_to_joints(&self, pose: &[JointTransform]) -> Option<Vec<Mat4>> {
        if pose.len() != self.joints.len() {
            return None;
        }

        // This is assuming the parent is always traversed before the child nodes

        // In model space
        let mut parent_transforms: Vec<Option<Mat4>> = vec![None; self.joints.len()];

        let mut transforms = vec![Mat4::IDENTITY; self.joints.len()];

        for i in 0..self.joints.len() {
            let joint = &self.joints[i];

            // In Bone space
            let current_local_transform: Mat4 = pose[i].into();
            // Convert to model space
            let current_transform = if joint.parent.is_none() {
                current_local_transform
            } else {
                parent_transforms[i]
                    .expect("Parent transform not set, joints may be incorrectly ordered.")
                    * current_local_transform
            };

            for c in joint.children.iter() {
                // Set parent transforms
                parent_transforms[*c as usize] = Some(current_transform);
            }

            transforms[i] = current_transform * joint.inverse_bind_matrix;
        }

        Some(transforms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
