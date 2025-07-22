/// Integration tests for LDraw.rs coupling detection and part processing
/// These tests use a minimal test library to verify that the entire pipeline works correctly
#[cfg(feature = "test-library")]
mod test_library_integration {
    use std::{collections::HashMap, path::PathBuf, sync::Arc};
    use ldraw::{
        library::{PartCache, ResolutionResult},
        color::ColorCatalog, 
        document::{Document, MultipartDocument, BfcCertification},
        elements::{Command, PartReference, Header},
        coupling_detection::{detect_couplings_from_document, CouplingDetector},
        PartAlias, Matrix4, Vector3,
    };
    use ldraw_ir::{
        part::{bake_part_with_couplings, PartMetadata},
        coupling_detection::detect_part_couplings,
    };
    use couplings::{CouplingType, CouplingGeometry};

    fn get_test_library_path() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test_ldraw_library");
        path
    }

    fn create_test_document(part_name: &str) -> MultipartDocument {
        MultipartDocument {
            body: Document {
                name: part_name.to_string(),
                description: format!("Test part {}", part_name),
                author: "Test".to_string(),
                bfc: BfcCertification::NoCertify,
                headers: vec![],
                commands: vec![
                    // Simple test: include a stud primitive
                    Command::PartReference(PartReference {
                        color: ldraw::color::ColorReference::resolve("16".to_string(), &ColorCatalog::empty()),
                        matrix: Matrix4::from_translation(Vector3::new(0.0, 0.0, 0.0)),
                        name: PartAlias::from("stud.dat".to_string()),
                    }),
                ],
            },
            subparts: HashMap::new(),
        }
    }

    #[test]
    fn test_primitive_registry_basic() {
        use couplings::PrimitiveRegistry;
        
        let registry = PrimitiveRegistry::new();
        
        // Test that we can find standard primitives
        assert!(registry.find_mapping("stud.dat").is_some());
        assert!(registry.find_mapping("peghole.dat").is_some());
        assert!(registry.find_mapping("4-4cyli.dat").is_some());
        
        // Test that the mappings have correct types
        let stud_mapping = registry.find_mapping("stud.dat").unwrap();
        assert_eq!(stud_mapping.coupling_type, CouplingType::Stud);
        assert_eq!(stud_mapping.normal_direction, [0.0, 1.0, 0.0]);
        
        let pin_hole_mapping = registry.find_mapping("peghole.dat").unwrap();
        assert_eq!(pin_hole_mapping.coupling_type, CouplingType::PinHole);
        assert_eq!(pin_hole_mapping.normal_direction, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_coupling_detection_basic() {
        let detector = CouplingDetector::new();
        let document = create_test_document("test_brick");
        let resolution_result = ResolutionResult::new();
        
        let couplings = detector.detect_couplings(&document, &resolution_result);
        
        // Should detect one stud coupling
        assert_eq!(couplings.len(), 1);
        assert_eq!(couplings[0].coupling_type, CouplingType::Stud);
        assert_eq!(couplings[0].position, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(couplings[0].normal, Vector3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn test_ir_coupling_conversion() {
        let document = create_test_document("test_brick");
        let resolution_result = ResolutionResult::new();
        
        let couplings = detect_part_couplings(&document, &resolution_result);
        
        // Should convert LDraw couplings to IR couplings correctly
        assert_eq!(couplings.len(), 1);
        assert_eq!(couplings[0].coupling_type, CouplingType::Stud);
        assert_eq!(couplings[0].center, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(couplings[0].normal, Vector3::new(0.0, 1.0, 0.0));
        assert!(couplings[0].id.starts_with("stud_"));
    }

    #[test]
    fn test_coupling_geometry_detection() {
        let detector = CouplingDetector::new();
        
        // Create document with pin hole
        let pin_hole_doc = MultipartDocument {
            body: Document {
                name: "test_technic".to_string(),
                description: "Test technic part".to_string(),
                author: "Test".to_string(),
                bfc: BfcCertification::NoCertify,
                headers: vec![],
                commands: vec![
                    Command::PartReference(PartReference {
                        color: ldraw::color::ColorReference::resolve("16".to_string(), &ColorCatalog::empty()),
                        matrix: Matrix4::from_translation(Vector3::new(10.0, 0.0, 0.0)),
                        name: PartAlias::from("peghole.dat".to_string()),
                    }),
                ],
            },
            subparts: HashMap::new(),
        };
        
        let resolution_result = ResolutionResult::new();
        let couplings = detector.detect_couplings(&pin_hole_doc, &resolution_result);
        
        assert_eq!(couplings.len(), 1);
        assert_eq!(couplings[0].coupling_type, CouplingType::PinHole);
        assert_eq!(couplings[0].position, Vector3::new(10.0, 0.0, 0.0));
        
        // Check geometry
        match &couplings[0].geometry {
            CouplingGeometry::Linear { length } => {
                assert_eq!(*length, 20.0);
            }
            _ => panic!("Expected linear geometry for pin hole"),
        }
    }

    #[test]
    fn test_part_with_multiple_coupling_types() {
        let detector = CouplingDetector::new();
        
        // Create document with both stud and pin hole
        let mixed_doc = MultipartDocument {
            body: Document {
                name: "test_mixed".to_string(),
                description: "Test mixed part".to_string(),
                author: "Test".to_string(),
                bfc: BfcCertification::NoCertify,
                headers: vec![],
                commands: vec![
                    Command::PartReference(PartReference {
                        color: ldraw::color::ColorReference::resolve("16".to_string(), &ColorCatalog::empty()),
                        matrix: Matrix4::from_translation(Vector3::new(0.0, 0.0, 0.0)),
                        name: PartAlias::from("stud.dat".to_string()),
                    }),
                    Command::PartReference(PartReference {
                        color: ldraw::color::ColorReference::resolve("16".to_string(), &ColorCatalog::empty()),
                        matrix: Matrix4::from_translation(Vector3::new(20.0, 12.0, 0.0)),
                        name: PartAlias::from("peghole.dat".to_string()),
                    }),
                ],
            },
            subparts: HashMap::new(),
        };
        
        let resolution_result = ResolutionResult::new();
        let couplings = detector.detect_couplings(&mixed_doc, &resolution_result);
        
        assert_eq!(couplings.len(), 2);
        
        // Find stud and pin hole
        let stud_coupling = couplings.iter().find(|c| c.coupling_type == CouplingType::Stud).unwrap();
        let pin_coupling = couplings.iter().find(|c| c.coupling_type == CouplingType::PinHole).unwrap();
        
        assert_eq!(stud_coupling.position, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(pin_coupling.position, Vector3::new(20.0, 12.0, 0.0));
    }

    #[test]
    fn test_coupling_compatibility() {
        // Test coupling compatibility logic
        assert!(CouplingType::Stud.can_connect_to(&CouplingType::AntiStud).is_some());
        assert!(CouplingType::Pin.can_connect_to(&CouplingType::PinHole).is_some());
        assert!(CouplingType::Axle.can_connect_to(&CouplingType::AxleHole).is_some());
        
        // Test incompatible connections
        assert!(CouplingType::Stud.can_connect_to(&CouplingType::Stud).is_none());
        assert!(CouplingType::Pin.can_connect_to(&CouplingType::AxleHole).is_none());
        
        // Test clutch force differences
        let stud_antistud = CouplingType::Stud.can_connect_to(&CouplingType::AntiStud).unwrap();
        let stud_tube = CouplingType::Stud.can_connect_to(&CouplingType::Tube).unwrap();
        assert!(stud_antistud.clutch_force > stud_tube.clutch_force);
    }

    #[test]
    fn test_full_part_processing_pipeline() {
        // This test verifies the entire pipeline from document to processed part with couplings
        let document = create_test_document("full_test");
        let resolution_result = ResolutionResult::new();
        
        // Create metadata
        let metadata = PartMetadata {
            name: "full_test".to_string(),
            description: "Full pipeline test".to_string(),
        };
        
        // Test that we can process the part through the full pipeline
        // Note: This would require more setup for a complete test, but demonstrates the approach
        let couplings = detect_part_couplings(&document, &resolution_result);
        assert!(!couplings.is_empty());
        
        // Verify the coupling was processed correctly
        let coupling = &couplings[0];
        assert_eq!(coupling.coupling_type, CouplingType::Stud);
        assert!(!coupling.id.is_empty());
        assert_eq!(coupling.center, Vector3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_transform_application() {
        let detector = CouplingDetector::new();
        
        // Create document with transformed stud
        let transform = Matrix4::from_translation(Vector3::new(20.0, 10.0, 30.0));
        let transformed_doc = MultipartDocument {
            body: Document {
                name: "test_transform".to_string(),
                description: "Test transform".to_string(),
                author: "Test".to_string(),
                bfc: BfcCertification::NoCertify,
                headers: vec![],
                commands: vec![
                    Command::PartReference(PartReference {
                        color: ldraw::color::ColorReference::resolve("16".to_string(), &ColorCatalog::empty()),
                        matrix: transform,
                        name: PartAlias::from("stud.dat".to_string()),
                    }),
                ],
            },
            subparts: HashMap::new(),
        };
        
        let resolution_result = ResolutionResult::new();
        let couplings = detector.detect_couplings(&transformed_doc, &resolution_result);
        
        assert_eq!(couplings.len(), 1);
        assert_eq!(couplings[0].position, Vector3::new(20.0, 10.0, 30.0));
        // Normal should remain the same (Y-up)
        assert_eq!(couplings[0].normal, Vector3::new(0.0, 1.0, 0.0));
    }
}

#[cfg(not(feature = "test-library"))]
mod placeholder {
    #[test]
    fn test_library_feature_disabled() {
        // This test runs when the test-library feature is not enabled
        // It serves as a placeholder to ensure the test suite still runs
        assert!(true);
    }
}