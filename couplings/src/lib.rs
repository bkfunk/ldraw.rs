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

/// LDraw Unit (LDU) constants for coupling geometry
/// 1 LDU = 0.4mm in real world
/// 20 LDU = 8mm = 1 standard LEGO module
pub mod ldu_constants {
    /// Full-length Technic pin or hole (1.0 modules = 20 LDU)
    pub const FULL_PIN_LENGTH: f32 = 20.0;
    /// Half-length Technic pin or hole (0.5 modules = 10 LDU)
    pub const HALF_PIN_LENGTH: f32 = 10.0;
    /// Axle segment length (0.2 modules = 4 LDU)
    pub const AXLE_SEGMENT_LENGTH: f32 = 4.0;
    /// Standard anti-stud tube inner radius (6 LDU)
    pub const TUBE_RADIUS: f32 = 6.0;
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
        self.register_stud_primitives();
        self.register_pin_primitives();
        self.register_friction_pin_primitives();
        self.register_pin_hole_primitives();
        self.register_axle_primitives();
        self.register_tube_primitives();
    }

    /// Register stud primitives
    fn register_stud_primitives(&mut self) {
        // Standard solid studs
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

        // Hollow studs (stud4.dat is context-dependent, see anti-stud detection)
        // For now, we register it as HollowStud; the detection code can override
        self.register_mapping(PrimitiveMapping {
            pattern: "stud4.dat".to_string(),
            coupling_type: CouplingType::HollowStud,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 1.0, 0.0],
            geometry_override: None,
        });

        self.register_mapping(PrimitiveMapping {
            pattern: "stud4a.dat".to_string(),
            coupling_type: CouplingType::HollowStud,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 1.0, 0.0],
            geometry_override: None,
        });
    }

    /// Register smooth (frictionless) pin primitives
    fn register_pin_primitives(&mut self) {
        use ldu_constants::*;

        // Full-length pins (1.0 = 20 LDU)
        for variant in ["connect.dat", "connect2.dat", "connect5.dat", "connect6.dat", "connect7.dat", "connect8.dat", "connect10.dat"] {
            self.register_mapping(PrimitiveMapping {
                pattern: variant.to_string(),
                coupling_type: CouplingType::Pin,
                position_offset: [0.0, 0.0, 0.0],
                normal_direction: [0.0, 0.0, 1.0],
                geometry_override: Some(CouplingGeometry::Linear { length: FULL_PIN_LENGTH }),
            });
        }

        // Half-length pins (0.5 = 10 LDU)
        for variant in ["connect3.dat", "connect4.dat"] {
            self.register_mapping(PrimitiveMapping {
                pattern: variant.to_string(),
                coupling_type: CouplingType::HalfPin,
                position_offset: [0.0, 0.0, 0.0],
                normal_direction: [0.0, 0.0, 1.0],
                geometry_override: Some(CouplingGeometry::Linear { length: HALF_PIN_LENGTH }),
            });
        }

        // Bushings (act like pins)
        self.register_mapping(PrimitiveMapping {
            pattern: "bush.dat".to_string(),
            coupling_type: CouplingType::Pin,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 0.0, 1.0],
            geometry_override: Some(CouplingGeometry::Linear { length: HALF_PIN_LENGTH }),
        });
    }

    /// Register friction pin primitives
    fn register_friction_pin_primitives(&mut self) {
        // Friction pins have ridges for grip
        // Full-length friction pins (1.0 = 20 LDU)
        self.register_mapping(PrimitiveMapping {
            pattern: "confric.dat".to_string(),
            coupling_type: CouplingType::PinWithFriction,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 0.0, 1.0],
            geometry_override: Some(CouplingGeometry::Linear { length: 20.0 }),
        });

        self.register_mapping(PrimitiveMapping {
            pattern: "confric2.dat".to_string(),
            coupling_type: CouplingType::PinWithFriction,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 0.0, 1.0],
            geometry_override: Some(CouplingGeometry::Linear { length: 20.0 }),
        });

        self.register_mapping(PrimitiveMapping {
            pattern: "confric5.dat".to_string(),
            coupling_type: CouplingType::PinWithFriction,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 0.0, 1.0],
            geometry_override: Some(CouplingGeometry::Linear { length: 20.0 }),
        });
    }

    /// Register pin hole primitives
    fn register_pin_hole_primitives(&mut self) {
        // Full pin holes
        self.register_mapping(PrimitiveMapping {
            pattern: "peghole.dat".to_string(),
            coupling_type: CouplingType::FullPinHole,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 0.0, 1.0],
            geometry_override: Some(CouplingGeometry::Linear { length: 20.0 }),
        });

        self.register_mapping(PrimitiveMapping {
            pattern: "peghole2.dat".to_string(),
            coupling_type: CouplingType::FullPinHole,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 0.0, 1.0],
            geometry_override: Some(CouplingGeometry::Linear { length: 20.0 }),
        });

        // Connector holes (similar to peghole)
        self.register_mapping(PrimitiveMapping {
            pattern: "connhole.dat".to_string(),
            coupling_type: CouplingType::FullPinHole,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 0.0, 1.0],
            geometry_override: Some(CouplingGeometry::Linear { length: 20.0 }),
        });

        // Half pin holes
        self.register_mapping(PrimitiveMapping {
            pattern: "peghole3.dat".to_string(),
            coupling_type: CouplingType::HalfPinHole,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 0.0, 1.0],
            geometry_override: Some(CouplingGeometry::Linear { length: 10.0 }),
        });

        // Negative pin holes (represent the hole space)
        self.register_mapping(PrimitiveMapping {
            pattern: "npeghole.dat".to_string(),
            coupling_type: CouplingType::FullPinHole,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 0.0, 1.0],
            geometry_override: Some(CouplingGeometry::Linear { length: 20.0 }),
        });

        // Beam holes (in Technic beams)
        self.register_mapping(PrimitiveMapping {
            pattern: "beamhole.dat".to_string(),
            coupling_type: CouplingType::FullPinHole,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 0.0, 1.0],
            geometry_override: Some(CouplingGeometry::Linear { length: 20.0 }),
        });
    }

    /// Register axle and axle hole primitives
    fn register_axle_primitives(&mut self) {
        // Axle cross-sections
        self.register_mapping(PrimitiveMapping {
            pattern: "axle.dat".to_string(),
            coupling_type: CouplingType::Axle,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 0.0, 1.0],
            geometry_override: Some(CouplingGeometry::Linear { length: 4.0 }),
        });

        self.register_mapping(PrimitiveMapping {
            pattern: "axleend.dat".to_string(),
            coupling_type: CouplingType::Axle,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 0.0, 1.0],
            geometry_override: Some(CouplingGeometry::Point),
        });

        // Axle holes
        self.register_mapping(PrimitiveMapping {
            pattern: "axlehole.dat".to_string(),
            coupling_type: CouplingType::AxleHole,
            position_offset: [0.0, 0.0, 0.0],
            normal_direction: [0.0, 0.0, 1.0],
            geometry_override: Some(CouplingGeometry::Linear { length: 20.0 }),
        });

        // Open axle holes (various depths)
        for i in 0..=11 {
            self.register_mapping(PrimitiveMapping {
                pattern: format!("axlehol{}.dat", i),
                coupling_type: CouplingType::AxleHole,
                position_offset: [0.0, 0.0, 0.0],
                normal_direction: [0.0, 0.0, 1.0],
                geometry_override: Some(CouplingGeometry::Linear { length: 20.0 }),
            });
        }
    }

    /// Register tube primitives (for anti-stud connections)
    fn register_tube_primitives(&mut self) {
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
    /// Matches primitives by exact name or by path suffix (e.g., "p/stud.dat" matches "stud.dat")
    pub fn find_mapping(&self, primitive_name: &str) -> Option<&PrimitiveMapping> {
        let name_lower = primitive_name.to_lowercase();
        self.mappings
            .iter()
            .find(|mapping| {
                let pattern = mapping.pattern.to_lowercase();
                // Match exact name or path suffix (handles both / and \ separators)
                name_lower == pattern
                    || name_lower.ends_with(&format!("/{}", pattern))
                    || name_lower.ends_with(&format!("\\{}", pattern))
            })
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

    #[test]
    fn test_stud_primitives() {
        let registry = PrimitiveRegistry::new();

        // Test solid studs
        let stud = registry.find_mapping("stud.dat").unwrap();
        assert_eq!(stud.coupling_type, CouplingType::Stud);
        assert_eq!(stud.normal_direction, [0.0, 1.0, 0.0]);

        let stud2 = registry.find_mapping("stud2.dat").unwrap();
        assert_eq!(stud2.coupling_type, CouplingType::Stud);

        // Test hollow studs
        let stud4 = registry.find_mapping("stud4.dat").unwrap();
        assert_eq!(stud4.coupling_type, CouplingType::HollowStud);

        let stud4a = registry.find_mapping("stud4a.dat").unwrap();
        assert_eq!(stud4a.coupling_type, CouplingType::HollowStud);
    }

    #[test]
    fn test_pin_primitives() {
        let registry = PrimitiveRegistry::new();

        // Test full-length pins
        let connect = registry.find_mapping("connect.dat").unwrap();
        assert_eq!(connect.coupling_type, CouplingType::Pin);
        assert_eq!(connect.normal_direction, [0.0, 0.0, 1.0]);
        assert!(matches!(
            connect.geometry_override,
            Some(CouplingGeometry::Linear { length }) if length == 20.0
        ));

        let connect2 = registry.find_mapping("connect2.dat").unwrap();
        assert_eq!(connect2.coupling_type, CouplingType::Pin);

        // Test half-length pins
        let connect3 = registry.find_mapping("connect3.dat").unwrap();
        assert_eq!(connect3.coupling_type, CouplingType::HalfPin);
        assert!(matches!(
            connect3.geometry_override,
            Some(CouplingGeometry::Linear { length }) if length == 10.0
        ));

        // Test bush (acts like pin)
        let bush = registry.find_mapping("bush.dat").unwrap();
        assert_eq!(bush.coupling_type, CouplingType::Pin);
    }

    #[test]
    fn test_friction_pin_primitives() {
        let registry = PrimitiveRegistry::new();

        let confric = registry.find_mapping("confric.dat").unwrap();
        assert_eq!(confric.coupling_type, CouplingType::PinWithFriction);
        assert_eq!(confric.normal_direction, [0.0, 0.0, 1.0]);
        assert!(matches!(
            confric.geometry_override,
            Some(CouplingGeometry::Linear { length }) if length == 20.0
        ));

        let confric2 = registry.find_mapping("confric2.dat").unwrap();
        assert_eq!(confric2.coupling_type, CouplingType::PinWithFriction);
    }

    #[test]
    fn test_pin_hole_primitives() {
        let registry = PrimitiveRegistry::new();

        // Test full pin holes
        let peghole = registry.find_mapping("peghole.dat").unwrap();
        assert_eq!(peghole.coupling_type, CouplingType::FullPinHole);
        assert_eq!(peghole.normal_direction, [0.0, 0.0, 1.0]);
        assert!(matches!(
            peghole.geometry_override,
            Some(CouplingGeometry::Linear { length }) if length == 20.0
        ));

        let connhole = registry.find_mapping("connhole.dat").unwrap();
        assert_eq!(connhole.coupling_type, CouplingType::FullPinHole);

        // Test half pin holes
        let peghole3 = registry.find_mapping("peghole3.dat").unwrap();
        assert_eq!(peghole3.coupling_type, CouplingType::HalfPinHole);
        assert!(matches!(
            peghole3.geometry_override,
            Some(CouplingGeometry::Linear { length }) if length == 10.0
        ));

        // Test negative pin holes
        let npeghole = registry.find_mapping("npeghole.dat").unwrap();
        assert_eq!(npeghole.coupling_type, CouplingType::FullPinHole);

        // Test beam holes
        let beamhole = registry.find_mapping("beamhole.dat").unwrap();
        assert_eq!(beamhole.coupling_type, CouplingType::FullPinHole);
    }

    #[test]
    fn test_axle_primitives() {
        let registry = PrimitiveRegistry::new();

        // Test axle
        let axle = registry.find_mapping("axle.dat").unwrap();
        assert_eq!(axle.coupling_type, CouplingType::Axle);
        assert_eq!(axle.normal_direction, [0.0, 0.0, 1.0]);

        let axleend = registry.find_mapping("axleend.dat").unwrap();
        assert_eq!(axleend.coupling_type, CouplingType::Axle);

        // Test axle holes
        let axlehole = registry.find_mapping("axlehole.dat").unwrap();
        assert_eq!(axlehole.coupling_type, CouplingType::AxleHole);
        assert_eq!(axlehole.normal_direction, [0.0, 0.0, 1.0]);

        // Test axle hole variants
        let axlehol0 = registry.find_mapping("axlehol0.dat").unwrap();
        assert_eq!(axlehol0.coupling_type, CouplingType::AxleHole);

        let axlehol5 = registry.find_mapping("axlehol5.dat").unwrap();
        assert_eq!(axlehol5.coupling_type, CouplingType::AxleHole);
    }

    #[test]
    fn test_tube_primitives() {
        let registry = PrimitiveRegistry::new();

        let tube = registry.find_mapping("4-4cyli.dat").unwrap();
        assert_eq!(tube.coupling_type, CouplingType::Tube);
        assert_eq!(tube.normal_direction, [0.0, 1.0, 0.0]);
        assert!(matches!(
            tube.geometry_override,
            Some(CouplingGeometry::Circular { radius }) if radius == 6.0
        ));
    }

    #[test]
    fn test_pin_to_pinhole_compatibility() {
        // Test smooth pin compatibility
        let props = CouplingType::Pin.can_connect_to(&CouplingType::FullPinHole);
        assert!(props.is_some());
        let props = props.unwrap();
        assert!(matches!(props.mode, ConnectionMode::Rotational { .. }));
        assert_eq!(props.clutch_force, 2.0); // Low for smooth rotation

        // Test friction pin compatibility
        let props = CouplingType::PinWithFriction.can_connect_to(&CouplingType::FullPinHole);
        assert!(props.is_some());
        let props = props.unwrap();
        assert!(matches!(props.mode, ConnectionMode::Rotational { .. }));
        assert_eq!(props.clutch_force, 12.0); // High for friction
    }

    #[test]
    fn test_axle_to_axlehole_compatibility() {
        let props = CouplingType::Axle.can_connect_to(&CouplingType::AxleHole);
        assert!(props.is_some());
        let props = props.unwrap();
        assert!(matches!(props.mode, ConnectionMode::RotationalLinear { .. }));

        // Axles can also fit in pin holes (loose fit)
        let props = CouplingType::Axle.can_connect_to(&CouplingType::FullPinHole);
        assert!(props.is_some());
    }

    #[test]
    fn test_registry_count() {
        let registry = PrimitiveRegistry::new();
        let count = registry.mappings().len();

        // We should have:
        // - 4 stud variants
        // - 8 pin variants (7 connect + 1 bush)
        // - 3 friction pin variants
        // - 7 pin hole variants
        // - 14 axle variants (2 axle + 1 axlehole + 11 axlehol*)
        // - 1 tube
        // Total: 37 primitives minimum
        assert!(count >= 37, "Expected at least 37 primitives, got {}", count);
    }

    #[test]
    fn test_primitive_matching_exact() {
        let registry = PrimitiveRegistry::new();

        // Should match exact name
        let stud = registry.find_mapping("stud.dat");
        assert!(stud.is_some());
        assert_eq!(stud.unwrap().pattern, "stud.dat");
    }

    #[test]
    fn test_primitive_case_insensitive() {
        let registry = PrimitiveRegistry::new();

        // All case variations should match
        assert!(registry.find_mapping("STUD.DAT").is_some());
        assert!(registry.find_mapping("Stud.Dat").is_some());
        assert!(registry.find_mapping("stud.dat").is_some());
        assert!(registry.find_mapping("StUd.DaT").is_some());
    }

    #[test]
    fn test_primitive_with_path_prefix() {
        let registry = PrimitiveRegistry::new();

        // Should match with path prefixes
        assert!(registry.find_mapping("p/stud.dat").is_some());
        assert!(registry.find_mapping("p\\stud.dat").is_some());
        assert!(registry.find_mapping("ldraw/p/stud.dat").is_some());

        // Verify it's the correct mapping
        let mapping = registry.find_mapping("p/stud.dat").unwrap();
        assert_eq!(mapping.coupling_type, CouplingType::Stud);
    }

    #[test]
    fn test_primitive_no_false_matches() {
        let registry = PrimitiveRegistry::new();

        // Should NOT match substrings in middle of name
        let result = registry.find_mapping("mystud.dat");
        // If found, it should NOT be the stud.dat mapping
        if let Some(mapping) = result {
            assert_ne!(mapping.pattern, "stud.dat",
                "mystud.dat should not match stud.dat pattern");
        }

        // Should NOT match partial names
        assert!(registry.find_mapping("stud").is_none());
        assert!(registry.find_mapping("stu.dat").is_none());
    }

    #[test]
    fn test_ambiguous_primitive_names() {
        let registry = PrimitiveRegistry::new();

        // axle.dat and daxle.dat are different primitives
        // Should only match exact or suffix, not substring
        let axle = registry.find_mapping("axle.dat");
        assert!(axle.is_some());
        assert_eq!(axle.unwrap().coupling_type, CouplingType::Axle);

        // "connect.dat" should not be confused with "connect2.dat"
        let connect = registry.find_mapping("connect.dat");
        let connect2 = registry.find_mapping("connect2.dat");
        assert!(connect.is_some());
        assert!(connect2.is_some());
        assert_eq!(connect.unwrap().pattern, "connect.dat");
        assert_eq!(connect2.unwrap().pattern, "connect2.dat");
    }

    #[test]
    fn test_all_primitives_have_valid_geometry() {
        let registry = PrimitiveRegistry::new();

        for mapping in registry.mappings() {
            // Check if there's a geometry override, otherwise get default
            let geometry = match &mapping.geometry_override {
                Some(geom) => geom,
                None => &mapping.coupling_type.default_geometry(),
            };

            // Verify geometry has valid values
            match geometry {
                CouplingGeometry::Linear { length } => {
                    assert!(*length > 0.0,
                        "Primitive {} has invalid length: {}",
                        mapping.pattern, length);
                }
                CouplingGeometry::Circular { radius } => {
                    assert!(*radius > 0.0,
                        "Primitive {} has invalid radius: {}",
                        mapping.pattern, radius);
                }
                _ => {}
            }
        }
    }
}
