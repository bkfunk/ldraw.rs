//! Coupling system for LEGO-style connections
//!
//! This crate provides:
//! - Coupling type definitions
//! - Physical connection properties
//! - Geometric analysis for coupling detection
//! - Mapping between LDraw primitives and coupling types

use serde::{Deserialize, Serialize};

/// Types of coupling connections available
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CouplingType {
    // Basic brick connections
    Stud,
    HollowStud,
    AntiStud,
    /// An anti-stud with a 1x1 round shape
    /// For example, seen in a 1x1 round plate.
    /// This can connect to a stud or be squeezed into the space between 4 studs arranged in a sqaure.
    Round1x1AntiStud,
    /// A projection that fits into a hollow stud's hole.
    /// These are found on the bottom of many 1x plates.
    Post,

    /// The point in the middle of a 4x4 stud grid.
    /// This can connect to the bottom of a 1x1 round plate, a full Technic bushing with four cutouts, etc.
    StudSquareCenter,

    // Technic connections
    Tube,
    FullPinHole,
    HalfPinHole,
    PlatePinHole,
    Pin,
    PinWithFriction,
    HalfPin,
    AxleHole,
    Axle,
    StandardGearToothMale,
    StandardGearToothFemale,
    BevelGearToothMale,
    BevelGearToothFemale,
    CrownGearToothMale,
    CrownGearToothFemale,
    WormGearToothMale,
    WormGearToothFemale,
    ScrewGearToothMale,
    ScrewGearToothFemale,
    HalfBushingToothMale,
    HalfBushingToothFemale,
    /// A full Technic bushing has one side that is round, and another that has 4 lobes and 4 cutouts; this lobed end can connect to the center of a 4x4 stud grid.
    BushingLobes,

    // Bar and clip connections
    BarSide,
    BarEnd,
    Clip,
    ClipOpen,

    // Hinge connections
    Hinge1x2BrickMale,
    Hinge1x2BrickFemale,
    VerticalHingePlate3Fingers,
    VerticalHingePlate2Fingers,
    BallJoint,
    BallSocket,

    // Rotating connections
    Turntable2x2Top,
    Turntable2x2Bottom,
    Turntable4x4Top,
    Turntable4x4Bottom,
    // TODO: Add more turntables
    HorizontalHingePlateTop,
    HorizontalHingePlateBottom,
    HelicopterRotorMale,
    HelicopterRotorFemale,
    IntegratedWheelAxle,
    IntegratedWheelHole,
    SmallWheelMale,
    SmallWheelFemale,

    // Rail connections
    Rail,
    RailGroove,

    // Chain connections
    ChainLinkMale,
    ChainLinkFemale,
    ChainLinkGearToothSlot,

    // Windows and doors
    StationaryWindowFemale,
    StationaryWindowMale,
    RotatingWindowFemale,
    RotatingWindowMale,
    DoorMale,
    DoorFemale,
}

/// How two couplings can connect and move relative to each other
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ConnectionMode {
    /// Fixed connection (e.g., stud to anti-stud)
    Fixed,
    /// Can rotate around the connection axis
    Rotational { friction: f32 },
    /// Can slide along the connection axis (e.g. a sliding plate)
    Linear { friction: f32 },
    /// Can both rotate and slide (e.g. an axle in a hole)
    RotationalLinear {
        rotational_friction: f32,
        linear_friction: f32,
    },
    /// Hinge with limited range
    Hinge {
        min_angle: f32,
        max_angle: f32,
        friction: f32,
    },
    /// Ball joint with angular limits
    BallJoint { max_angle: f32, friction: f32 },
}

/// Physical properties of a specific connection between two coupling types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ConnectionProperties {
    /// How the parts can move relative to each other
    pub mode: ConnectionMode,
    /// Force required to break the connection (in Newtons)
    pub break_force: f32,
    /// Torque required to break the connection (in Newton-meters)
    pub break_torque: f32,
    /// Whether this connection can be made/broken during simulation
    pub dynamic: bool,
    /// Clutch force - resistance to initial movement
    pub clutch_force: f32,
}

/// Geometric constraint for valid connection positions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CouplingGeometry {
    /// Single point connection (e.g., stud)
    Point,
    /// Linear connection along an axis (e.g., axle hole)
    Linear { length: f32 },
    /// Circular connection (e.g., turntable)
    Circular { radius: f32 },
    /// Rectangular area (e.g., large plate connections)
    Rectangular { width: f32, height: f32 },
    /// Complex geometric space (e.g., antistud area)
    GeometricSpace {
        /// Bounding box of the space
        bounds: BoundingBox,
        /// Whether the space is convex
        convex: bool,
    },
}

/// Simple bounding box for geometric spaces
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub min_z: f32,
    pub max_z: f32,
}

/// Source of a coupling - how it was detected
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CouplingSource {
    /// Detected from an LDraw primitive
    Primitive {
        /// Name of the primitive file
        primitive_name: String,
        /// Transform applied to the primitive
        transform: [f32; 16], // 4x4 matrix as flat array
    },
    /// Detected from geometric analysis
    GeometricAnalysis {
        /// Description of the analysis method
        method: String,
        /// Confidence level (0.0 to 1.0)
        confidence: f32,
    },
    /// Manually specified
    Manual,
}

impl CouplingType {
    /// Returns human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            CouplingType::AntiStud => "Anti-stud",
            // Fallback: generate name from variant identifier
            _ => {
                // Get the variant name as a string
                let variant = format!("{:?}", self);
                // Insert spaces before capital letters and numbers, and replace underscores with spaces
                let mut name = String::new();
                let mut prev_is_lower = false;
                for c in variant.chars() {
                    if c == '_' {
                        name.push(' ');
                        prev_is_lower = false;
                    } else if c.is_uppercase() && prev_is_lower {
                        name.push(' ');
                        name.push(c);
                        prev_is_lower = false;
                    } else if c.is_numeric() && prev_is_lower {
                        name.push(' ');
                        name.push(c);
                        prev_is_lower = false;
                    } else {
                        name.push(c);
                        prev_is_lower = c.is_lowercase();
                    }
                }
                Box::leak(name.into_boxed_str())
            }
        }
    }

    /// Check if this coupling type can connect to another, returning connection properties
    pub fn can_connect_to(&self, other: &CouplingType) -> Option<ConnectionProperties> {
        use ConnectionMode::*;
        use CouplingType::*;

        match (self, other) {
            // Stud connections - different properties based on coupling types
            (Stud, AntiStud) | (AntiStud, Stud) => Some(ConnectionProperties {
                mode: Fixed,
                break_force: 10.0,
                break_torque: 5.0,
                dynamic: true,
                clutch_force: 8.0, // High clutch for standard connection
            }),

            // Stud to tube connection - different properties
            (Stud, Tube) | (Tube, Stud) => Some(ConnectionProperties {
                mode: Fixed,
                break_force: 12.0,
                break_torque: 6.0,
                dynamic: true,
                clutch_force: 6.0, // Lower clutch than stud-antistud
            }),

            // Hollow stud connections
            (HollowStud, AntiStud) | (AntiStud, HollowStud) => Some(ConnectionProperties {
                mode: Fixed,
                break_force: 8.0,
                break_torque: 4.0,
                dynamic: true,
                clutch_force: 6.0,
            }),

            // Pin connections
            (Pin, FullPinHole) | (FullPinHole, Pin) => Some(ConnectionProperties {
                mode: Rotational { friction: 0.1 },
                break_force: 15.0,
                break_torque: 8.0,
                dynamic: true,
                clutch_force: 2.0, // Low clutch for smooth rotation
            }),
            (PinWithFriction, FullPinHole) | (FullPinHole, PinWithFriction) => {
                Some(ConnectionProperties {
                    mode: Rotational { friction: 2.0 },
                    break_force: 15.0,
                    break_torque: 8.0,
                    dynamic: true,
                    clutch_force: 12.0, // High clutch for friction pins
                })
            }

            // Axle connections
            (Axle, AxleHole) | (AxleHole, Axle) => Some(ConnectionProperties {
                mode: RotationalLinear {
                    rotational_friction: 0.05,
                    linear_friction: 1.0,
                },
                break_force: 12.0,
                break_torque: 10.0,
                dynamic: true,
                clutch_force: 1.0, // Very low clutch for smooth operation
            }),
            (Axle, FullPinHole) | (FullPinHole, Axle) => Some(ConnectionProperties {
                mode: RotationalLinear {
                    rotational_friction: 0.2,
                    linear_friction: 2.0,
                },
                break_force: 8.0,
                break_torque: 6.0,
                dynamic: true,
                clutch_force: 3.0, // Moderate clutch for loose fit
            }),

            // Bar and clip
            (BarEnd, Clip) | (Clip, BarEnd) => Some(ConnectionProperties {
                mode: Fixed,
                break_force: 5.0,
                break_torque: 3.0,
                dynamic: true,
                clutch_force: 4.0,
            }),
            (BarSide, ClipOpen) | (ClipOpen, BarSide) => Some(ConnectionProperties {
                mode: Rotational { friction: 0.5 },
                break_force: 4.0,
                break_torque: 2.5,
                dynamic: true,
                clutch_force: 2.0,
            }),

            // Hinge connections
            (Hinge1x2BrickMale, Hinge1x2BrickFemale) | (Hinge1x2BrickFemale, Hinge1x2BrickMale) => {
                Some(ConnectionProperties {
                    mode: Hinge {
                        min_angle: -180.0,
                        max_angle: 180.0,
                        friction: 0.3,
                    },
                    break_force: 10.0,
                    break_torque: 12.0,
                    dynamic: false,
                    clutch_force: 5.0,
                })
            }

            // Ball joint
            (BallJoint, BallSocket) | (BallSocket, BallJoint) => Some(ConnectionProperties {
                mode: ConnectionMode::BallJoint {
                    max_angle: 45.0,
                    friction: 0.4,
                },
                break_force: 8.0,
                break_torque: 10.0,
                dynamic: false,
                clutch_force: 6.0,
            }),

            // Rail connections
            (Rail, RailGroove) | (RailGroove, Rail) => Some(ConnectionProperties {
                mode: Linear { friction: 0.3 },
                break_force: 20.0,
                break_torque: 5.0,
                dynamic: true,
                clutch_force: 3.0,
            }),

            _ => None,
        }
    }

    /// Get the expected geometry for this coupling type
    pub fn default_geometry(&self) -> CouplingGeometry {
        match self {
            CouplingType::Stud | CouplingType::HollowStud => CouplingGeometry::Point,
            CouplingType::AntiStud => CouplingGeometry::GeometricSpace {
                bounds: BoundingBox {
                    min_x: -10.0,
                    max_x: 10.0,
                    min_y: -4.0,
                    max_y: 0.0,
                    min_z: -10.0,
                    max_z: 10.0,
                },
                convex: true,
            },
            CouplingType::FullPinHole | CouplingType::AxleHole => {
                CouplingGeometry::Linear { length: 20.0 }
            }
            CouplingType::Tube => CouplingGeometry::Circular { radius: 6.0 },
            _ => CouplingGeometry::Point,
        }
    }
}

/// Mapping from LDraw primitive names to coupling types
pub struct PrimitiveMapping {
    /// Pattern to match primitive names (case-insensitive)
    pub pattern: String,
    /// Coupling type this primitive represents
    pub coupling_type: CouplingType,
    /// Local position offset from primitive origin
    pub position_offset: [f32; 3],
    /// Local normal direction from primitive origin
    pub normal_direction: [f32; 3],
    /// Override default geometry if needed
    pub geometry_override: Option<CouplingGeometry>,
}

/// Registry of standard LDraw primitive mappings
pub struct PrimitiveRegistry {
    mappings: Vec<PrimitiveMapping>,
}

impl Default for PrimitiveRegistry {
    fn default() -> Self {
        let mut registry = Self {
            mappings: Vec::new(),
        };
        registry.register_standard_primitives();
        registry
    }
}

impl PrimitiveRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register standard LDraw primitives
    fn register_standard_primitives(&mut self) {
        // Standard studs
        self.register_mapping(PrimitiveMapping {
            pattern: "stud.dat".to_string(),
            coupling_type: CouplingType::Stud,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 1.0, 0.0],
            geometry_override: None,
        });

        self.register_mapping(PrimitiveMapping {
            pattern: "stud2.dat".to_string(),
            coupling_type: CouplingType::Stud,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 1.0, 0.0],
            geometry_override: None,
        });

        // Hollow studs
        self.register_mapping(PrimitiveMapping {
            pattern: "stud4.dat".to_string(),
            coupling_type: CouplingType::HollowStud,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 1.0, 0.0],
            geometry_override: None,
        });

        // Technic holes
        self.register_mapping(PrimitiveMapping {
            pattern: "peghole.dat".to_string(),
            coupling_type: CouplingType::FullPinHole,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 0.0, 1.0],
            geometry_override: None,
        });

        self.register_mapping(PrimitiveMapping {
            pattern: "axlehole.dat".to_string(),
            coupling_type: CouplingType::AxleHole,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 0.0, 1.0],
            geometry_override: None,
        });

        // Tubes
        self.register_mapping(PrimitiveMapping {
            pattern: "4-4cyli.dat".to_string(),
            coupling_type: CouplingType::Tube,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 1.0, 0.0],
            geometry_override: Some(CouplingGeometry::Circular { radius: 6.0 }),
        });
    }

    pub fn register_mapping(&mut self, mapping: PrimitiveMapping) {
        self.mappings.push(mapping);
    }

    /// Find mapping for a primitive name
    pub fn find_mapping(&self, primitive_name: &str) -> Option<&PrimitiveMapping> {
        let name_lower = primitive_name.to_lowercase();
        self.mappings
            .iter()
            .find(|mapping| name_lower.contains(&mapping.pattern.to_lowercase()))
    }

    /// Get all registered mappings
    pub fn mappings(&self) -> &[PrimitiveMapping] {
        &self.mappings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coupling_compatibility() {
        // Test stud to antistud
        let props = CouplingType::Stud.can_connect_to(&CouplingType::AntiStud);
        assert!(props.is_some());
        let props = props.unwrap();
        assert_eq!(props.clutch_force, 8.0);

        // Test stud to tube - different properties
        let props = CouplingType::Stud.can_connect_to(&CouplingType::Tube);
        assert!(props.is_some());
        let props = props.unwrap();
        assert_eq!(props.clutch_force, 6.0); // Lower than stud-antistud
    }

    #[test]
    fn test_primitive_registry() {
        let registry = PrimitiveRegistry::new();

        let mapping = registry.find_mapping("stud.dat");
        assert!(mapping.is_some());
        let mapping = mapping.unwrap();
        assert_eq!(mapping.coupling_type, CouplingType::Stud);
    }

    #[test]
    fn test_antistud_geometry() {
        let geometry = CouplingType::AntiStud.default_geometry();
        assert!(matches!(geometry, CouplingGeometry::GeometricSpace { .. }));
    }
}
