/// Coupling detection from LDraw primitives and geometry
/// This bridges the gap between ldraw's coupling detection and IR's coupling types
use ldraw::{
    coupling_detection::{detect_couplings_from_document, DetectedCoupling},
    document::MultipartDocument,
    library::ResolutionResult,
};

use crate::{
    coupling::Coupling,
    geometry::BoundingBox3,
};

/// Convert a detected coupling from ldraw to IR coupling
fn convert_detected_coupling(detected: &DetectedCoupling, id: usize) -> Coupling {
    let coupling_type = detected.coupling_type;
    let geometry = detected.geometry.clone();

    Coupling {
        coupling_type,
        id: format!(
            "{}_{}",
            coupling_type.name().to_lowercase().replace(' ', "_"),
            id
        ),
        center: detected.position,
        normal: detected.normal,
        geometry,
        excludes: Vec::new(),
    }
}

/// Detect couplings from a multipart document and convert to IR couplings
pub fn detect_part_couplings(
    document: &MultipartDocument,
    resolution_result: &ResolutionResult,
) -> Vec<Coupling> {
    let detected_couplings = detect_couplings_from_document(document, resolution_result);

    detected_couplings
        .iter()
        .enumerate()
        .map(|(i, detected)| convert_detected_coupling(detected, i))
        .collect()
}

/// Detect couplings from part geometry.
///
/// This function is intended for future implementation, where it will analyze the provided
/// geometry and metadata to infer coupling patterns automatically, replacing any hardcoded
/// dimension extraction currently used elsewhere.
///
/// # Arguments
/// * `_bounding_box` - The bounding box of the part geometry.
/// * `_part_name` - The name of the part.
/// * `_description` - The description of the part.
///
/// # Returns
/// An empty vector for now, as this is a placeholder for future logic.
#[allow(dead_code)]
pub fn detect_from_geometry(
    _bounding_box: &BoundingBox3,
    _part_name: &str,
    _description: &str,
) -> Vec<Coupling> {
    // Future implementation: analyze geometry to infer coupling patterns.
    // This would replace any hardcoded dimension extraction.
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coupling_type_names() {
        assert_eq!(CouplingType::Stud.name(), "Stud");
        assert_eq!(CouplingType::FullPinHole.name(), "Pin Hole");
        assert_eq!(CouplingType::AxleHole.name(), "Axle Hole");
    }

    #[test]
    fn test_geometry_types() {
        // Test linear geometry
        let linear_geometry = CouplingGeometry::Linear { length: 20.0 };
        assert!(matches!(linear_geometry, CouplingGeometry::Linear { length } if length == 20.0));

        // Test circular geometry
        let circular_geometry = CouplingGeometry::Circular { radius: 6.0 };
        assert!(
            matches!(circular_geometry, CouplingGeometry::Circular { radius } if radius == 6.0)
        );

        // Test point geometry
        let point_geometry = CouplingGeometry::Point;
        assert!(matches!(point_geometry, CouplingGeometry::Point));
    }
}
