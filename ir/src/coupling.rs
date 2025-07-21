/// A Coupling is a point where one part can be connected to another.
use cgmath::InnerSpace;
use ldraw::Vector3;

// Re-export types from couplings crate
pub use couplings::{CouplingType, ConnectionMode, ConnectionProperties, CouplingGeometry};

type Normal = Vector3;
type Point = Vector3;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
            CouplingGeometry::GeometricSpace { .. } => {
                // TODO: Implement geometric space validation
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
        assert_eq!(CouplingType::FullPinHole.name(), "Pin Hole");
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
        let props = CouplingType::Pin.can_connect_to(&CouplingType::FullPinHole);
        assert!(props.is_some());
        if let Some(props) = props {
            assert!(matches!(props.mode, ConnectionMode::Rotational { friction } if friction == 0.1));
        }

        // Friction pin in pin hole
        let props = CouplingType::PinWithFriction.can_connect_to(&CouplingType::FullPinHole);
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
        let props = CouplingType::Axle.can_connect_to(&CouplingType::FullPinHole);
        assert!(props.is_some());
    }

    #[test]
    fn test_incompatible_connections() {
        // Stud cannot connect to stud
        assert!(CouplingType::Stud.can_connect_to(&CouplingType::Stud).is_none());
        
        // Pin cannot connect to axle hole
        assert!(CouplingType::Pin.can_connect_to(&CouplingType::AxleHole).is_none());
        
        // Bar cannot connect to pin hole
        assert!(CouplingType::Bar.can_connect_to(&CouplingType::FullPinHole).is_none());
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
            coupling_type: CouplingType::FullPinHole,
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