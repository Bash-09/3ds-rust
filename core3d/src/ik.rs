use glam::{Mat4, Vec3};

use crate::{JointTransform, Skeleton};

pub struct IKSolver {
    skeleton: Skeleton,
    joint_transforms: Vec<JointTransform>,
}

impl IKSolver {
    pub fn new(skeleton: Skeleton) -> Self {
        // In model space
        let mut parent_transforms: Vec<Option<Mat4>> = vec![None; skeleton.joints.len()];

        let mut joint_transforms = vec![JointTransform::default(); skeleton.joints.len()];

        for i in 0..skeleton.joints.len() {
            let joint = &skeleton.joints[i];

            // In Bone space
            let current_local_transform: Mat4 = skeleton.joints[i].base_transform.into();
            // Convert to model space
            let current_transform = if joint.parent.is_none() {
                current_local_transform
            } else {
                parent_transforms[i].expect("Parent transform not set") * current_local_transform
            };

            for c in joint.children.iter() {
                // Set parent transforms
                parent_transforms[*c as usize] = Some(current_transform);
            }

            joint_transforms[i] = current_transform.into();
        }

        Self {
            skeleton,
            joint_transforms,
        }
    }

    /// Returns the transform for each joint in model space. Applying the transform for a specific joint to the origin vector (0.0, 0.0, 0.0, 1.0) will yield the position of that joint in model space.
    pub fn joint_transforms_model_space(&self) -> &[JointTransform] {
        &self.joint_transforms
    }

    pub fn solve_joint(
        &mut self,
        source_joint: Option<usize>,
        target_joint: usize,
        target_pos: Vec3,
    ) {
        const ITERATIONS: usize = 3;
    }
}
