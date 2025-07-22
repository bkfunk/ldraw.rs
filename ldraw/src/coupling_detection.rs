use crate::{document::MultipartDocument, elements::Command, library::ResolutionResult, Vector3};
/// Coupling detection from LDraw document structure
/// This module operates on the parsed LDraw document structure before IR conversion
use cgmath::{InnerSpace, Point3, SquareMatrix, Transform};
use couplings::{CouplingGeometry, CouplingSource, CouplingType, PrimitiveRegistry};

/// Represents a detected coupling with its world-space transform
#[derive(Debug, Clone)]
pub struct DetectedCoupling {
    /// Type of coupling detected
    pub coupling_type: CouplingType,
    /// World-space position of the coupling
    pub position: Vector3,
    /// World-space normal direction of the coupling
    pub normal: Vector3,
    /// Geometry of the coupling
    pub geometry: CouplingGeometry,
    /// Source of the coupling detection
    pub source: CouplingSource,
}

/// Main coupling detection interface
pub struct CouplingDetector {
    registry: PrimitiveRegistry,
}

impl Default for CouplingDetector {
    fn default() -> Self {
        Self {
            registry: PrimitiveRegistry::default(),
        }
    }
}

impl CouplingDetector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Detect couplings from a multipart document
    pub fn detect_couplings(
        &self,
        document: &MultipartDocument,
        resolution_result: &ResolutionResult,
    ) -> Vec<DetectedCoupling> {
        let mut couplings = Vec::new();

        // Process the main document body
        self.process_commands(
            &document.body.commands,
            &crate::Matrix4::identity(),
            resolution_result,
            &mut couplings,
        );

        // Process all resolved subparts
        for (_, subpart_doc) in &document.subparts {
            self.process_commands(
                &subpart_doc.commands,
                &crate::Matrix4::identity(),
                resolution_result,
                &mut couplings,
            );
        }

        couplings
    }

    /// Process LDraw commands recursively
    fn process_commands(
        &self,
        commands: &[Command],
        transform: &crate::Matrix4,
        resolution_result: &ResolutionResult,
        couplings: &mut Vec<DetectedCoupling>,
    ) {
        for command in commands {
            match command {
                Command::PartReference(part_ref) => {
                    // Combine transformations
                    let combined_transform = transform * part_ref.matrix;

                    // Check if this part reference is a coupling primitive
                    if let Some(mapping) = self.registry.find_mapping(&part_ref.name.normalized) {
                        let coupling = self.create_coupling_from_mapping(
                            mapping,
                            &combined_transform,
                            &part_ref.name.normalized,
                        );
                        couplings.push(coupling);
                    } else {
                        // If it's not a primitive, check if it's a resolved part with its own commands
                        if let Some((resolved_doc, _is_local)) =
                            resolution_result.query(&part_ref.name, false)
                        {
                            self.process_commands(
                                &resolved_doc.body.commands,
                                &combined_transform,
                                resolution_result,
                                couplings,
                            );
                        }
                    }
                }
                // We could extend this to handle other command types if needed
                _ => {}
            }
        }
    }

    /// Create a coupling from a mapping and transformation
    fn create_coupling_from_mapping(
        &self,
        mapping: &couplings::PrimitiveMapping,
        transform: &crate::Matrix4,
        source_primitive: &str,
    ) -> DetectedCoupling {
        // Transform the position and normal from local to world space
        let local_position = Point3::new(
            mapping.position_offset[0],
            mapping.position_offset[1],
            mapping.position_offset[2],
        );
        let world_position = transform.transform_point(local_position);
        let position = Vector3::new(world_position.x, world_position.y, world_position.z);

        let local_normal = Vector3::new(
            mapping.normal_direction[0],
            mapping.normal_direction[1],
            mapping.normal_direction[2],
        );
        let world_normal = transform.transform_vector(local_normal).normalize();

        // Use geometry override if provided, otherwise use default
        let geometry = mapping
            .geometry_override
            .clone()
            .unwrap_or_else(|| mapping.coupling_type.default_geometry());

        // Create transform array for source tracking
        let transform_array = [
            transform[0][0],
            transform[0][1],
            transform[0][2],
            transform[0][3],
            transform[1][0],
            transform[1][1],
            transform[1][2],
            transform[1][3],
            transform[2][0],
            transform[2][1],
            transform[2][2],
            transform[2][3],
            transform[3][0],
            transform[3][1],
            transform[3][2],
            transform[3][3],
        ];

        DetectedCoupling {
            coupling_type: mapping.coupling_type,
            position,
            normal: world_normal,
            geometry,
            source: CouplingSource::Primitive {
                primitive_name: source_primitive.to_string(),
                transform: transform_array,
            },
        }
    }
}

/// Convenience function to detect couplings from a document
pub fn detect_couplings_from_document(
    document: &MultipartDocument,
    resolution_result: &ResolutionResult,
) -> Vec<DetectedCoupling> {
    let detector = CouplingDetector::new();
    detector.detect_couplings(document, resolution_result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        color::ColorReference, elements::PartReference, library::ResolutionResult, PartAlias,
    };
    use std::collections::HashMap;

    #[test]
    fn test_primitive_registry() {
        let registry = PrimitiveRegistry::new();

        // Test stud mapping
        let stud_mapping = registry.find_mapping("stud.dat");
        assert!(stud_mapping.is_some());
        let mapping = stud_mapping.unwrap();
        assert_eq!(mapping.coupling_type, CouplingType::Stud);
        assert_eq!(mapping.normal_direction, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_coupling_detector() {
        let detector = CouplingDetector::new();

        // Create a simple test document
        let part_ref = PartReference {
            color: ColorReference::Current,
            matrix: crate::Matrix4::from_scale(1.0),
            name: PartAlias::from("stud.dat".to_string()),
        };

        let document = MultipartDocument {
            body: crate::document::Document {
                name: "test".to_string(),
                description: "Test part".to_string(),
                author: "Test".to_string(),
                bfc: crate::document::BfcCertification::NoCertify,
                headers: Vec::new(),
                commands: vec![Command::PartReference(part_ref)],
            },
            subparts: HashMap::new(),
        };

        let resolution_result = ResolutionResult::new();
        let couplings = detector.detect_couplings(&document, &resolution_result);

        assert_eq!(couplings.len(), 1);
        assert_eq!(couplings[0].coupling_type, CouplingType::Stud);
        assert_eq!(couplings[0].position, Vector3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_pin_hole_detection() {
        let detector = CouplingDetector::new();

        let part_ref = PartReference {
            color: ColorReference::Current,
            matrix: crate::Matrix4::from_translation(Vector3::new(10.0, 0.0, 0.0)),
            name: PartAlias::from("peghole.dat".to_string()),
        };

        let document = MultipartDocument {
            body: crate::document::Document {
                name: "test".to_string(),
                description: "Test part".to_string(),
                author: "Test".to_string(),
                bfc: crate::document::BfcCertification::NoCertify,
                headers: Vec::new(),
                commands: vec![Command::PartReference(part_ref)],
            },
            subparts: HashMap::new(),
        };

        let resolution_result = ResolutionResult::new();
        let couplings = detector.detect_couplings(&document, &resolution_result);

        assert_eq!(couplings.len(), 1);
        assert_eq!(couplings[0].coupling_type, CouplingType::FullPinHole);
        assert_eq!(couplings[0].position, Vector3::new(10.0, 0.0, 0.0));

        // Check that it has linear geometry
        match &couplings[0].geometry {
            CouplingGeometry::Linear { length } => {
                assert_eq!(*length, 20.0);
            }
            _ => panic!("Expected linear geometry for pin hole"),
        }
    }
}
