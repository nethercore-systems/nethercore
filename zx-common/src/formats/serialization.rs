//! Binary serialization trait for format headers.
//!
//! All Nethercore ZX format headers implement `BinarySerializable` for
//! consistent serialization/deserialization. This provides a unified interface
//! for generic code while each header retains its type-specific `to_bytes()`
//! method returning a fixed-size array for efficiency.

/// Trait for binary-serializable format headers.
///
/// All Nethercore ZX format headers implement this trait for consistent
/// serialization/deserialization. The trait uses `Vec<u8>` for the return type
/// because associated const generics in return types (`[u8; Self::SIZE]`) are
/// not yet stable in Rust.
///
/// For performance-critical code, use the type-specific `to_bytes()` methods
/// directly, which return fixed-size arrays.
///
/// # Example
///
/// ```
/// use zx_common::formats::{BinarySerializable, NetherZXTextureHeader};
///
/// let header = NetherZXTextureHeader::new(64, 64);
///
/// // Using the trait (returns Vec<u8>)
/// let bytes = header.serialize();
/// let parsed = NetherZXTextureHeader::deserialize(&bytes).unwrap();
///
/// // Using the type-specific method (returns [u8; 4])
/// let bytes_array = header.to_bytes();
/// ```
pub trait BinarySerializable: Sized {
    /// Size of the serialized header in bytes.
    const SIZE: usize;

    /// Serialize to bytes.
    ///
    /// Returns a `Vec<u8>` containing the serialized representation.
    /// For a fixed-size array, use the type-specific `to_bytes()` method.
    fn serialize(&self) -> Vec<u8>;

    /// Deserialize from bytes.
    ///
    /// Returns `None` if the byte slice is too short or contains invalid data.
    fn deserialize(bytes: &[u8]) -> Option<Self>;
}

// Implementation for NetherZXMeshHeader
impl BinarySerializable for super::NetherZXMeshHeader {
    const SIZE: usize = Self::SIZE;

    fn serialize(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }

    fn deserialize(bytes: &[u8]) -> Option<Self> {
        Self::from_bytes(bytes)
    }
}

// Implementation for NetherZXTextureHeader
impl BinarySerializable for super::NetherZXTextureHeader {
    const SIZE: usize = Self::SIZE;

    fn serialize(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }

    fn deserialize(bytes: &[u8]) -> Option<Self> {
        Self::from_bytes(bytes)
    }
}

// Implementation for NetherZXSkeletonHeader
impl BinarySerializable for super::NetherZXSkeletonHeader {
    const SIZE: usize = Self::SIZE;

    fn serialize(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }

    fn deserialize(bytes: &[u8]) -> Option<Self> {
        Self::from_bytes(bytes)
    }
}

// Implementation for NetherZXSoundHeader
impl BinarySerializable for super::NetherZXSoundHeader {
    const SIZE: usize = Self::SIZE;

    fn serialize(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }

    fn deserialize(bytes: &[u8]) -> Option<Self> {
        Self::from_bytes(bytes)
    }
}

// Implementation for NetherZXAnimationHeader
impl BinarySerializable for super::NetherZXAnimationHeader {
    const SIZE: usize = Self::SIZE;

    fn serialize(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }

    fn deserialize(bytes: &[u8]) -> Option<Self> {
        Self::from_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::{
        NetherZXAnimationHeader, NetherZXMeshHeader, NetherZXSkeletonHeader, NetherZXSoundHeader,
        NetherZXTextureHeader,
    };

    #[test]
    fn test_mesh_header_trait() {
        let header = NetherZXMeshHeader::new(100, 300, 0x07);
        let bytes = header.serialize();
        assert_eq!(bytes.len(), NetherZXMeshHeader::SIZE);
        assert_eq!(<NetherZXMeshHeader as BinarySerializable>::SIZE, 12);

        let parsed = NetherZXMeshHeader::deserialize(&bytes).unwrap();
        assert_eq!(parsed.vertex_count, 100);
        assert_eq!(parsed.index_count, 300);
        assert_eq!(parsed.format, 0x07);
    }

    #[test]
    fn test_texture_header_trait() {
        let header = NetherZXTextureHeader::new(256, 128);
        let bytes = header.serialize();
        assert_eq!(bytes.len(), NetherZXTextureHeader::SIZE);
        assert_eq!(<NetherZXTextureHeader as BinarySerializable>::SIZE, 4);

        let parsed = NetherZXTextureHeader::deserialize(&bytes).unwrap();
        assert_eq!(parsed.width, 256);
        assert_eq!(parsed.height, 128);
    }

    #[test]
    fn test_skeleton_header_trait() {
        let header = NetherZXSkeletonHeader::new(24);
        let bytes = header.serialize();
        assert_eq!(bytes.len(), NetherZXSkeletonHeader::SIZE);
        assert_eq!(<NetherZXSkeletonHeader as BinarySerializable>::SIZE, 8);

        let parsed = NetherZXSkeletonHeader::deserialize(&bytes).unwrap();
        assert_eq!(parsed.bone_count, 24);
    }

    #[test]
    fn test_sound_header_trait() {
        let header = NetherZXSoundHeader::new(44100);
        let bytes = header.serialize();
        assert_eq!(bytes.len(), NetherZXSoundHeader::SIZE);
        assert_eq!(<NetherZXSoundHeader as BinarySerializable>::SIZE, 8);

        let parsed = NetherZXSoundHeader::deserialize(&bytes).unwrap();
        assert_eq!(parsed.total_samples, 44100);
    }

    #[test]
    fn test_animation_header_trait() {
        let header = NetherZXAnimationHeader::new(32, 120);
        let bytes = header.serialize();
        assert_eq!(bytes.len(), NetherZXAnimationHeader::SIZE);
        assert_eq!(<NetherZXAnimationHeader as BinarySerializable>::SIZE, 4);

        let parsed = NetherZXAnimationHeader::deserialize(&bytes).unwrap();
        assert_eq!(parsed.bone_count, 32);
        assert_eq!(parsed.frame_count, 120);
    }

    #[test]
    fn test_deserialize_insufficient_bytes() {
        // All headers should return None when given insufficient bytes
        assert!(NetherZXMeshHeader::deserialize(&[0; 11]).is_none());
        assert!(NetherZXTextureHeader::deserialize(&[0; 3]).is_none());
        assert!(NetherZXSkeletonHeader::deserialize(&[0; 7]).is_none());
        assert!(NetherZXSoundHeader::deserialize(&[0; 7]).is_none());
        assert!(NetherZXAnimationHeader::deserialize(&[0; 3]).is_none());
    }

    /// Demonstrates generic function using the trait
    fn header_size<T: BinarySerializable>() -> usize {
        T::SIZE
    }

    #[test]
    fn test_generic_usage() {
        assert_eq!(header_size::<NetherZXMeshHeader>(), 12);
        assert_eq!(header_size::<NetherZXTextureHeader>(), 4);
        assert_eq!(header_size::<NetherZXSkeletonHeader>(), 8);
        assert_eq!(header_size::<NetherZXSoundHeader>(), 8);
        assert_eq!(header_size::<NetherZXAnimationHeader>(), 4);
    }
}
