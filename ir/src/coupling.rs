/// A Coupling is a point where one part can be connected to another.
use cgmath::InnerSpace;
use ldraw::Vector3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CouplingType {
    // Basic brick connections
    Stud,
    HollowStud,
    AntiStud,
    
    // Technic connections
    Tube,
    PinHole,
    Pin,
    PinWithFriction,
    AxleHole,
    Axle,
    
    // Bar and clip connections
    Bar,
    Clip,
    ClipOpen,  // Clips that can be opened/closed
    
    // Hinge connections
    HingeMale,
    HingeFemale,
    BallJoint,
    BallSocket,
    
    // Rail connections
    Rail,
    RailGroove,
}

/// Defines how two couplings can connect
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionMode {
    /// Fixed connection (e.g., stud to anti-stud)
    Fixed,
    /// Can rotate around the connection axis
    Rotational { friction: f32 },
    /// Can slide along the connection axis
    Linear { friction: f32 },
    /// Can both rotate and slide
    RotationalLinear { rotational_friction: f32, linear_friction: f32 },
    /// Hinge with limited range
    Hinge { min_angle: f32, max_angle: f32, friction: f32 },
    /// Ball joint with angular limits
    BallJoint { max_angle: f32, friction: f32 },
}

/// Physical properties of a connection
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConnectionProperties {
    /// How the parts can move relative to each other
    pub mode: ConnectionMode,
    /// Force required to break the connection (in Newtons)
    pub break_force: f32,
    /// Torque required to break the connection (in Newton-meters)
    pub break_torque: f32,
    /// Whether this connection can be made/broken during simulation
    pub dynamic: bool,
}

impl CouplingType {
    /// Returns human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            CouplingType::Stud => "Stud",
            CouplingType::HollowStud => "Hollow Stud",
            CouplingType::AntiStud => "Anti-stud",
            CouplingType::Tube => "Tube",
            CouplingType::PinHole => "Pin Hole",
            CouplingType::Pin => "Pin",
            CouplingType::PinWithFriction => "Pin with Friction",
            CouplingType::AxleHole => "Axle Hole",
            CouplingType::Axle => "Axle",
            CouplingType::Bar => "Bar",
            CouplingType::Clip => "Clip",
            CouplingType::ClipOpen => "Open Clip",
            CouplingType::HingeMale => "Hinge Male",
            CouplingType::HingeFemale => "Hinge Female",
            CouplingType::BallJoint => "Ball Joint",
            CouplingType::BallSocket => "Ball Socket",
            CouplingType::Rail => "Rail",
            CouplingType::RailGroove => "Rail Groove",
        }
    }
    
    /// Check if this coupling type can connect to another
    pub fn can_connect_to(&self, other: &CouplingType) -> Option<ConnectionProperties> {
        use CouplingType::*;
        use ConnectionMode::*;
        
        match (self, other) {
            // Stud connections
            (Stud, AntiStud) | (AntiStud, Stud) => Some(ConnectionProperties {
                mode: Fixed,
                break_force: 10.0,
                break_torque: 5.0,
                dynamic: true,
            }),
            (HollowStud, AntiStud) | (AntiStud, HollowStud) => Some(ConnectionProperties {
                mode: Fixed,
                break_force: 8.0,
                break_torque: 4.0,
                dynamic: true,
            }),
            
            // Pin connections
            (Pin, PinHole) | (PinHole, Pin) => Some(ConnectionProperties {
                mode: Rotational { friction: 0.1 },
                break_force: 15.0,
                break_torque: 8.0,
                dynamic: true,
            }),
            (PinWithFriction, PinHole) | (PinHole, PinWithFriction) => Some(ConnectionProperties {
                mode: Rotational { friction: 2.0 },
                break_force: 15.0,
                break_torque: 8.0,
                dynamic: true,
            }),
            
            // Axle connections
            (Axle, AxleHole) | (AxleHole, Axle) => Some(ConnectionProperties {
                mode: RotationalLinear { 
                    rotational_friction: 0.05,
                    linear_friction: 1.0 
                },
                break_force: 12.0,
                break_torque: 10.0,
                dynamic: true,
            }),
            (Axle, PinHole) | (PinHole, Axle) => Some(ConnectionProperties {
                mode: RotationalLinear { 
                    rotational_friction: 0.2,
                    linear_friction: 2.0 
                },
                break_force: 8.0,
                break_torque: 6.0,
                dynamic: true,
            }),
            
            // Bar and clip
            (Bar, Clip) | (Clip, Bar) => Some(ConnectionProperties {
                mode: Fixed,
                break_force: 5.0,
                break_torque: 3.0,
                dynamic: true,
            }),
            (Bar, ClipOpen) | (ClipOpen, Bar) => Some(ConnectionProperties {
                mode: Rotational { friction: 0.5 },
                break_force: 4.0,
                break_torque: 2.5,
                dynamic: true,
            }),
            
            // Hinge connections
            (HingeMale, HingeFemale) | (HingeFemale, HingeMale) => Some(ConnectionProperties {
                mode: Hinge { 
                    min_angle: -180.0,
                    max_angle: 180.0,
                    friction: 0.3 
                },
                break_force: 10.0,
                break_torque: 12.0,
                dynamic: false,
            }),
            
            // Ball joint
            (CouplingType::BallJoint, CouplingType::BallSocket) | (CouplingType::BallSocket, CouplingType::BallJoint) => Some(ConnectionProperties {
                mode: ConnectionMode::BallJoint { 
                    max_angle: 45.0,
                    friction: 0.4 
                },
                break_force: 8.0,
                break_torque: 10.0,
                dynamic: false,
            }),
            
            // Rail connections
            (Rail, RailGroove) | (RailGroove, Rail) => Some(ConnectionProperties {
                mode: Linear { friction: 0.3 },
                break_force: 20.0,
                break_torque: 5.0,
                dynamic: true,
            }),
            
            _ => None,
        }
    }
}

type Normal = Vector3;
type Point = Vector3;

/// Geometric constraint for valid connection positions
#[derive(Debug, Clone, PartialEq)]
pub enum CouplingGeometry {
    /// Single point connection (e.g., stud)
    Point,
    /// Linear connection along an axis (e.g., axle hole)
    Linear { length: f32 },
    /// Circular connection (e.g., turntable)
    Circular { radius: f32 },
    /// Rectangular area (e.g., large plate connections)
    Rectangular { width: f32, height: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Coupling {
    /// The type of this coupling
    pub coupling_type: CouplingType,
    
    /// Unique identifier for this coupling within the part
    pub id: String,

    /// The center of the coupling point in part-local coordinates
    pub center: Point,
    
    /// The normal vector of the coupling (direction it faces)
    pub normal: Normal,
    
    /// Geometric constraints for valid connection positions
    pub geometry: CouplingGeometry,
    
    /// Optional: which other coupling IDs on the same part are incompatible
    /// (e.g., can't use both ends of a clip simultaneously)
    pub excludes: Vec<String>,
}

impl Coupling {
    /// Create a simple point coupling
    pub fn new_point(coupling_type: CouplingType, id: String, center: Point, normal: Normal) -> Self {
        Self {
            coupling_type,
            id,
            center,
            normal,
            geometry: CouplingGeometry::Point,
            excludes: Vec::new(),
        }
    }
    
    /// Check if a connection point is valid given the geometry
    pub fn is_valid_connection_point(&self, point: &Point) -> bool {
        match &self.geometry {
            CouplingGeometry::Point => {
                (point - self.center).magnitude() < 0.1
            }
            CouplingGeometry::Linear { length } => {
                let to_point = point - self.center;
                let projection = to_point.dot(self.normal);
                projection.abs() <= length / 2.0 && 
                    (to_point - self.normal * projection).magnitude() < 0.1
            }
            CouplingGeometry::Circular { radius } => {
                let to_point = point - self.center;
                let distance = (to_point - self.normal * to_point.dot(self.normal)).magnitude();
                (distance - radius).abs() < 0.1
            }
            CouplingGeometry::Rectangular { width: _, height: _ } => {
                // TODO: Implement rectangular geometry validation
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coupling_type_name() {
        assert_eq!(CouplingType::Stud.name(), "Stud");
        assert_eq!(CouplingType::AntiStud.name(), "Anti-stud");
        assert_eq!(CouplingType::PinHole.name(), "Pin Hole");
        assert_eq!(CouplingType::Axle.name(), "Axle");
    }

    #[test]
    fn test_stud_antistud_compatibility() {
        let props = CouplingType::Stud.can_connect_to(&CouplingType::AntiStud);
        assert!(props.is_some());
        let props = props.unwrap();
        assert_eq!(props.mode, ConnectionMode::Fixed);
        assert_eq!(props.break_force, 10.0);
        assert_eq!(props.break_torque, 5.0);
        assert!(props.dynamic);
    }

    #[test]
    fn test_pin_hole_compatibility() {
        // Pin in pin hole
        let props = CouplingType::Pin.can_connect_to(&CouplingType::PinHole);
        assert!(props.is_some());
        if let Some(props) = props {
            assert!(matches!(props.mode, ConnectionMode::Rotational { friction } if friction == 0.1));
        }

        // Friction pin in pin hole
        let props = CouplingType::PinWithFriction.can_connect_to(&CouplingType::PinHole);
        assert!(props.is_some());
        if let Some(props) = props {
            assert!(matches!(props.mode, ConnectionMode::Rotational { friction } if friction == 2.0));
        }
    }

    #[test]
    fn test_axle_compatibility() {
        // Axle in axle hole
        let props = CouplingType::Axle.can_connect_to(&CouplingType::AxleHole);
        assert!(props.is_some());
        if let Some(props) = props {
            assert!(matches!(props.mode, ConnectionMode::RotationalLinear { .. }));
        }

        // Axle in pin hole (loose fit)
        let props = CouplingType::Axle.can_connect_to(&CouplingType::PinHole);
        assert!(props.is_some());
    }

    #[test]
    fn test_incompatible_connections() {
        // Stud cannot connect to stud
        assert!(CouplingType::Stud.can_connect_to(&CouplingType::Stud).is_none());
        
        // Pin cannot connect to axle hole
        assert!(CouplingType::Pin.can_connect_to(&CouplingType::AxleHole).is_none());
        
        // Bar cannot connect to pin hole
        assert!(CouplingType::Bar.can_connect_to(&CouplingType::PinHole).is_none());
    }

    #[test]
    fn test_coupling_geometry_point() {
        let coupling = Coupling::new_point(
            CouplingType::Stud,
            "test_stud".to_string(),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(),
        );

        // Point at exact location
        assert!(coupling.is_valid_connection_point(&Vector3::new(0.0, 0.0, 0.0)));
        
        // Point slightly off
        assert!(coupling.is_valid_connection_point(&Vector3::new(0.05, 0.0, 0.0)));
        
        // Point too far
        assert!(!coupling.is_valid_connection_point(&Vector3::new(0.2, 0.0, 0.0)));
    }

    #[test]
    fn test_coupling_geometry_linear() {
        let coupling = Coupling {
            coupling_type: CouplingType::PinHole,
            id: "test_pin_hole".to_string(),
            center: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::unit_x(),
            geometry: CouplingGeometry::Linear { length: 20.0 },
            excludes: Vec::new(),
        };

        // Point on axis within length
        assert!(coupling.is_valid_connection_point(&Vector3::new(5.0, 0.0, 0.0)));
        assert!(coupling.is_valid_connection_point(&Vector3::new(-5.0, 0.0, 0.0)));
        
        // Point at limit
        assert!(coupling.is_valid_connection_point(&Vector3::new(10.0, 0.0, 0.0)));
        
        // Point beyond limit
        assert!(!coupling.is_valid_connection_point(&Vector3::new(15.0, 0.0, 0.0)));
        
        // Point off axis
        assert!(!coupling.is_valid_connection_point(&Vector3::new(5.0, 0.5, 0.0)));
    }

    #[test]
    fn test_coupling_geometry_circular() {
        let coupling = Coupling {
            coupling_type: CouplingType::Tube,
            id: "test_tube".to_string(),
            center: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::unit_y(),
            geometry: CouplingGeometry::Circular { radius: 6.0 },
            excludes: Vec::new(),
        };

        // Point on circle
        assert!(coupling.is_valid_connection_point(&Vector3::new(6.0, 0.0, 0.0)));
        assert!(coupling.is_valid_connection_point(&Vector3::new(0.0, 0.0, 6.0)));
        
        // Point slightly off circle
        assert!(coupling.is_valid_connection_point(&Vector3::new(5.95, 0.0, 0.0)));
        
        // Point too far from circle
        assert!(!coupling.is_valid_connection_point(&Vector3::new(7.0, 0.0, 0.0)));
    }
}
