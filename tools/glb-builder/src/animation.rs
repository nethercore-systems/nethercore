//! Animation track construction

use crate::buffer::{AccessorIndex, BufferBuilder};

/// Accessor indices for animation data
#[derive(Debug, Clone)]
pub struct AnimationAccessors {
    pub times: AccessorIndex,
    pub translations: Vec<AccessorIndex>,
    pub rotations: Vec<AccessorIndex>,
    pub scales: Vec<AccessorIndex>,
}

/// Builder for animation tracks
pub struct AnimationBuilder {
    times: Vec<f32>,
    translations: Vec<Vec<[f32; 3]>>,
    rotations: Vec<Vec<[f32; 4]>>,
    scales: Vec<Vec<[f32; 3]>>,
}

impl AnimationBuilder {
    /// Create new animation builder with specified bone count
    pub fn new(bone_count: usize) -> Self {
        Self {
            times: Vec::new(),
            translations: vec![Vec::new(); bone_count],
            rotations: vec![Vec::new(); bone_count],
            scales: vec![Vec::new(); bone_count],
        }
    }

    /// Set animation times (keyframes)
    pub fn times(mut self, times: &[f32]) -> Self {
        self.times = times.to_vec();
        self
    }

    /// Set translation track for a specific bone
    pub fn bone_translations(mut self, bone_idx: usize, translations: &[[f32; 3]]) -> Self {
        self.translations[bone_idx] = translations.to_vec();
        self
    }

    /// Set rotation track for a specific bone
    pub fn bone_rotations(mut self, bone_idx: usize, rotations: &[[f32; 4]]) -> Self {
        self.rotations[bone_idx] = rotations.to_vec();
        self
    }

    /// Set scale track for a specific bone
    pub fn bone_scales(mut self, bone_idx: usize, scales: &[[f32; 3]]) -> Self {
        self.scales[bone_idx] = scales.to_vec();
        self
    }

    /// Set all translation tracks at once
    pub fn all_translations(mut self, translations: Vec<Vec<[f32; 3]>>) -> Self {
        self.translations = translations;
        self
    }

    /// Set all rotation tracks at once
    pub fn all_rotations(mut self, rotations: Vec<Vec<[f32; 4]>>) -> Self {
        self.rotations = rotations;
        self
    }

    /// Set all scale tracks at once
    pub fn all_scales(mut self, scales: Vec<Vec<[f32; 3]>>) -> Self {
        self.scales = scales;
        self
    }

    /// Get bone count
    pub fn bone_count(&self) -> usize {
        self.translations.len()
    }

    /// Build and pack into buffer
    pub fn build(self, buffer: &mut BufferBuilder) -> AnimationAccessors {
        let times = buffer.pack_scalars_with_bounds(&self.times);

        let mut translation_accessors = Vec::new();
        for bone_trans in &self.translations {
            translation_accessors.push(buffer.pack_vec3(bone_trans));
        }

        let mut rotation_accessors = Vec::new();
        for bone_rot in &self.rotations {
            rotation_accessors.push(buffer.pack_vec4(bone_rot));
        }

        let mut scale_accessors = Vec::new();
        for bone_scale in &self.scales {
            scale_accessors.push(buffer.pack_vec3(bone_scale));
        }

        AnimationAccessors {
            times,
            translations: translation_accessors,
            rotations: rotation_accessors,
            scales: scale_accessors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_builder() {
        let mut buffer = BufferBuilder::new();
        let anim = AnimationBuilder::new(2)
            .times(&[0.0, 1.0])
            .bone_translations(0, &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]])
            .bone_translations(1, &[[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]])
            .bone_rotations(0, &[[0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, 1.0]])
            .bone_rotations(1, &[[0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, 1.0]])
            .bone_scales(0, &[[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]])
            .bone_scales(1, &[[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]])
            .build(&mut buffer);

        assert_eq!(anim.times, AccessorIndex(0));
        assert_eq!(anim.translations.len(), 2);
        assert_eq!(anim.rotations.len(), 2);
        assert_eq!(anim.scales.len(), 2);
    }
}
