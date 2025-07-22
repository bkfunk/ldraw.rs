/// Integration tests for ldraw-ir coupling processing
/// These tests verify the IR layer correctly processes couplings from the ldraw layer

#[cfg(feature = "test-library")]
mod ir_integration_tests {
    use std::collections::HashMap;
    use ldraw::{
        library::ResolutionResult,
        document::{Document, MultipartDocument, BfcCertification},
        elements::{Command, PartReference},
        PartAlias, Matrix4, Vector3,
    };
    use ldraw_ir::{
        coupling_detection::detect_part_couplings,
        coupling::{Coupling, CouplingType, CouplingGeometry},
    };

    fn create_test_document_with_primitives() -> MultipartDocument {
        MultipartDocument {
            body: Document {
                name: "test_part".to_string(),
                description: "Test part with multiple coupling types".to_string(),
                author: "Test".to_string(),
                bfc: BfcCertification::NoCertify,
                headers: vec![],
                commands: vec![
                    // Stud at origin
                    Command::PartReference(PartReference {
                        color: ldraw::color::ColorReference::resolve(16, &HashMap::new()),
                        matrix: Matrix4::from_translation(Vector3::new(0.0, 0.0, 0.0)),
                        name: PartAlias::from("stud.dat".to_string()),
                    }),
                    // Pin hole at different location
                    Command::PartReference(PartReference {
                        color: ldraw::color::ColorReference::resolve(16, &HashMap::new()),
                        matrix: Matrix4::from_translation(Vector3::new(20.0, 12.0, 0.0)),
                        name: PartAlias::from("peghole.dat".to_string()),
                    }),
                    // Tube (cylinder primitive)
                    Command::PartReference(PartReference {
                        color: ldraw::color::ColorReference::resolve(16, &HashMap::new()),
                        matrix: Matrix4::from_translation(Vector3::new(-10.0, 0.0, 20.0)),
                        name: PartAlias::from("4-4cyli.dat".to_string()),
                    }),
                ],
            },
            subparts: HashMap::new(),
        }
    }

    #[test]
    fn test_ir_coupling_detection_basic() {
        let document = create_test_document_with_primitives();
        let resolution_result = ResolutionResult::new();
        
        let couplings = detect_part_couplings(&document, &resolution_result);
        
        // Should detect three couplings: stud, pin hole, and tube
        assert_eq!(couplings.len(), 3);
        
        // Verify all expected types are present
        let coupling_types: Vec<CouplingType> = couplings.iter().map(|c| c.coupling_type).collect();
        assert!(coupling_types.contains(&CouplingType::Stud));
        assert!(coupling_types.contains(&CouplingType::PinHole));
        assert!(coupling_types.contains(&CouplingType::Tube));
    }

    #[test]
    fn test_ir_coupling_ids_generated() {
        let document = create_test_document_with_primitives();
        let resolution_result = ResolutionResult::new();
        
        let couplings = detect_part_couplings(&document, &resolution_result);
        
        // Check that all couplings have valid IDs
        for coupling in &couplings {
            assert!(!coupling.id.is_empty());
            
            // Check ID format matches expected pattern
            match coupling.coupling_type {
                CouplingType::Stud => assert!(coupling.id.starts_with("stud_")),
                CouplingType::PinHole => assert!(coupling.id.starts_with("pin_hole_")),
                CouplingType::Tube => assert!(coupling.id.starts_with("tube_")),
                _ => {} // Other types are okay
            }
        }
        
        // Check that all IDs are unique
        let mut ids: Vec<String> = couplings.iter().map(|c| c.id.clone()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), couplings.len());
    }

    #[test]
    fn test_ir_coupling_positions_transformed() {
        let document = create_test_document_with_primitives();
        let resolution_result = ResolutionResult::new();
        
        let couplings = detect_part_couplings(&document, &resolution_result);
        
        // Find each coupling type and verify positions
        let stud_coupling = couplings.iter().find(|c| c.coupling_type == CouplingType::Stud).unwrap();
        let pin_coupling = couplings.iter().find(|c| c.coupling_type == CouplingType::PinHole).unwrap();
        let tube_coupling = couplings.iter().find(|c| c.coupling_type == CouplingType::Tube).unwrap();
        
        // Check positions match the transformations applied
        assert_eq!(stud_coupling.center, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(pin_coupling.center, Vector3::new(20.0, 12.0, 0.0));
        assert_eq!(tube_coupling.center, Vector3::new(-10.0, 0.0, 20.0));
    }

    #[test]
    fn test_ir_coupling_normals() {
        let document = create_test_document_with_primitives();
        let resolution_result = ResolutionResult::new();
        
        let couplings = detect_part_couplings(&document, &resolution_result);
        
        let stud_coupling = couplings.iter().find(|c| c.coupling_type == CouplingType::Stud).unwrap();
        let pin_coupling = couplings.iter().find(|c| c.coupling_type == CouplingType::PinHole).unwrap();
        
        // Check normals are correct for each coupling type
        assert_eq!(stud_coupling.normal, Vector3::new(0.0, 1.0, 0.0)); // Y-up
        assert_eq!(pin_coupling.normal, Vector3::new(0.0, 0.0, 1.0));  // Z-forward
    }

    #[test]
    fn test_ir_coupling_geometry_types() {
        let document = create_test_document_with_primitives();
        let resolution_result = ResolutionResult::new();
        
        let couplings = detect_part_couplings(&document, &resolution_result);
        
        let stud_coupling = couplings.iter().find(|c| c.coupling_type == CouplingType::Stud).unwrap();
        let pin_coupling = couplings.iter().find(|c| c.coupling_type == CouplingType::PinHole).unwrap();
        let tube_coupling = couplings.iter().find(|c| c.coupling_type == CouplingType::Tube).unwrap();
        
        // Check geometry types
        assert!(matches!(stud_coupling.geometry, CouplingGeometry::Point));
        
        match &pin_coupling.geometry {
            CouplingGeometry::Linear { length } => {
                assert_eq!(*length, 20.0);
            }
            _ => panic!("Expected linear geometry for pin hole"),
        }
        
        match &tube_coupling.geometry {
            CouplingGeometry::Circular { radius } => {
                assert_eq!(*radius, 6.0);
            }
            _ => panic!("Expected circular geometry for tube"),
        }
    }

    #[test]
    fn test_ir_coupling_validation() {
        let document = create_test_document_with_primitives();
        let resolution_result = ResolutionResult::new();
        
        let couplings = detect_part_couplings(&document, &resolution_result);
        
        // Test geometry validation for each coupling
        for coupling in &couplings {
            match coupling.coupling_type {
                CouplingType::Stud => {
                    // Test point validation
                    assert!(coupling.is_valid_connection_point(&coupling.center));
                    assert!(coupling.is_valid_connection_point(&(coupling.center + Vector3::new(0.05, 0.0, 0.0))));
                    assert!(!coupling.is_valid_connection_point(&(coupling.center + Vector3::new(0.2, 0.0, 0.0))));
                }
                CouplingType::PinHole => {
                    // Test linear validation
                    let test_point = coupling.center + coupling.normal * 5.0;
                    assert!(coupling.is_valid_connection_point(&test_point));
                    
                    let too_far_point = coupling.center + coupling.normal * 15.0;
                    assert!(!coupling.is_valid_connection_point(&too_far_point));
                }
                _ => {} // Skip validation for other types in this test
            }
        }
    }

    #[test]
    fn test_ir_coupling_empty_excludes() {
        let document = create_test_document_with_primitives();
        let resolution_result = ResolutionResult::new();
        
        let couplings = detect_part_couplings(&document, &resolution_result);
        
        // For basic primitives, excludes should be empty
        for coupling in &couplings {
            assert!(coupling.excludes.is_empty());
        }
    }
}

#[cfg(not(feature = "test-library"))]
mod placeholder {
    #[test]
    fn test_ir_library_feature_disabled() {
        // This test runs when the test-library feature is not enabled
        assert!(true);
    }
}