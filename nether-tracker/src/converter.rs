//! Module conversion trait
//!
//! This module provides a trait abstraction for converting format-specific
//! modules (XM, IT) to the unified TrackerModule format.

use crate::TrackerModule;

/// Trait for converting format-specific modules to TrackerModule
///
/// This trait abstracts the common conversion pattern shared by XM and IT converters:
/// 1. Parse format-specific module
/// 2. Convert patterns, instruments, samples
/// 3. Build unified TrackerModule
///
/// # Type Parameters
///
/// - `Source`: The source module type (e.g., `nether_xm::XmModule`, `nether_it::ItModule`)
///
/// # Examples
///
/// Converting an XM module:
/// ```ignore
/// use nether_tracker::{ModuleConverter, XmConverter};
///
/// let xm_module = nether_xm::parse_xm(&data)?;
/// let tracker_module = XmConverter::convert(&xm_module);
/// ```
///
/// Converting an IT module:
/// ```ignore
/// use nether_tracker::{ModuleConverter, ItConverter};
///
/// let it_module = nether_it::parse_it(&data)?;
/// let tracker_module = ItConverter::convert(&it_module);
/// ```
pub trait ModuleConverter {
    /// The source module type to convert from
    type Source;

    /// Convert a source module to the unified TrackerModule format
    ///
    /// # Arguments
    ///
    /// * `source` - The source module to convert
    ///
    /// # Returns
    ///
    /// The converted TrackerModule (infallible for XM/IT)
    fn convert(source: &Self::Source) -> TrackerModule;
}

/// XM to TrackerModule converter
pub struct XmConverter;

impl ModuleConverter for XmConverter {
    type Source = nether_xm::XmModule;

    fn convert(source: &Self::Source) -> TrackerModule {
        crate::convert_xm::from_xm_module(source)
    }
}

/// IT to TrackerModule converter
pub struct ItConverter;

impl ModuleConverter for ItConverter {
    type Source = nether_it::ItModule;

    fn convert(source: &Self::Source) -> TrackerModule {
        crate::convert_it::from_it_module(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xm_converter_type() {
        // Verify the converter implements the trait with correct associated type
        fn assert_converter<T: ModuleConverter>() {}
        assert_converter::<XmConverter>();
    }

    #[test]
    fn test_it_converter_type() {
        fn assert_converter<T: ModuleConverter>() {}
        assert_converter::<ItConverter>();
    }
}
