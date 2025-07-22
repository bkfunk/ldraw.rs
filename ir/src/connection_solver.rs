use cgmath::{EuclideanSpace, InnerSpace, Transform};
use ldraw::{Matrix4, Point3, Vector3};

use crate::connection::Connection;
use crate::coupling::CouplingGeometry;

/// Helper to find valid connection points between two couplings
pub fn find_connection_points(
    coupling_a: &crate::coupling::Coupling,
    transform_a: &Matrix4,
    coupling_b: &crate::coupling::Coupling,
    transform_b: &Matrix4,
) -> Option<(Vector3, Vector3)> {
    // Transform coupling positions to world space
    let world_pos_a = transform_a
        .transform_point(Point3::from_vec(coupling_a.center))
        .to_vec();
    let world_normal_a = transform_a.transform_vector(coupling_a.normal).normalize();

    let world_pos_b = transform_b
        .transform_point(Point3::from_vec(coupling_b.center))
        .to_vec();
    let world_normal_b = transform_b.transform_vector(coupling_b.normal).normalize();

    // Check if normals are roughly opposite (required for most connections)
    let normal_dot = world_normal_a.dot(world_normal_b);
    if normal_dot > -0.8 {
        return None; // Normals not opposite enough
    }

    // Find connection points based on geometry types
    match (&coupling_a.geometry, &coupling_b.geometry) {
        (CouplingGeometry::Point, CouplingGeometry::Point) => {
            // Simple point-to-point
            Some((world_pos_a, world_pos_b))
        }
        (CouplingGeometry::Point, CouplingGeometry::Linear { length: _ })
        | (CouplingGeometry::Linear { length: _ }, CouplingGeometry::Point) => {
            // Project point onto line
            // For now, return the center points
            // TODO: Implement proper line projection
            Some((world_pos_a, world_pos_b))
        }
        (
            CouplingGeometry::Linear { length: _len_a },
            CouplingGeometry::Linear { length: _len_b },
        ) => {
            // Line to line connection
            // TODO: Find closest points between two line segments
            Some((world_pos_a, world_pos_b))
        }
        (CouplingGeometry::Circular { .. }, CouplingGeometry::Circular { .. }) => {
            // Circle to circle (e.g., turntables)
            Some((world_pos_a, world_pos_b))
        }
        _ => {
            // TODO: Handle other geometry combinations
            Some((world_pos_a, world_pos_b))
        }
    }
}

/// Calculate the stress on a connection given applied forces
pub fn calculate_connection_stress(
    connection: &Connection,
    force_a: Vector3,
    torque_a: Vector3,
    force_b: Vector3,
    torque_b: Vector3,
) -> (f32, f32) {
    // Net force on connection
    let net_force = force_a - force_b;
    let net_torque = torque_a - torque_b;

    // Calculate stress factors (0.0 = no stress, 1.0 = at breaking point)
    let force_stress = net_force.magnitude() / connection.properties.break_force;
    let torque_stress = net_torque.magnitude() / connection.properties.break_torque;

    (force_stress, torque_stress)
}

/// Check if two couplings are within connection distance
pub fn couplings_within_range(
    coupling_a: &crate::coupling::Coupling,
    transform_a: &Matrix4,
    coupling_b: &crate::coupling::Coupling,
    transform_b: &Matrix4,
    max_distance: f32,
) -> bool {
    let world_pos_a = transform_a
        .transform_point(Point3::from_vec(coupling_a.center))
        .to_vec();
    let world_pos_b = transform_b
        .transform_point(Point3::from_vec(coupling_b.center))
        .to_vec();

    (world_pos_b - world_pos_a).magnitude() <= max_distance
}

/// Project a point onto a line segment defined by coupling geometry
pub fn project_point_to_linear_coupling(
    point: Vector3,
    coupling_center: Vector3,
    coupling_normal: Vector3,
    length: f32,
) -> Vector3 {
    let to_point = point - coupling_center;
    let projection = to_point.dot(coupling_normal);
    let clamped = projection.clamp(-length / 2.0, length / 2.0);
    coupling_center + coupling_normal * clamped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coupling::{Coupling, CouplingGeometry, CouplingType};
    use cgmath::{SquareMatrix, Zero};
    use uuid::Uuid;

    #[test]
    fn test_find_connection_points_opposite_normals() {
        let coupling_a = Coupling::new_point(
            CouplingType::Stud,
            "stud".to_string(),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(),
        );
        
        let coupling_b = Coupling::new_point(
            CouplingType::AntiStud,
            "antistud".to_string(),
            Vector3::new(0.0, 10.0, 0.0),
            -Vector3::unit_y(),
        );
        
        let transform_a = Matrix4::identity();
        let transform_b = Matrix4::identity();
        
        let result = find_connection_points(&coupling_a, &transform_a, &coupling_b, &transform_b);
        assert!(result.is_some());
        
        let (point_a, point_b) = result.unwrap();
        assert_eq!(point_a, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(point_b, Vector3::new(0.0, 10.0, 0.0));
    }

    #[test]
    fn test_find_connection_points_non_opposite_normals() {
        let coupling_a = Coupling::new_point(
            CouplingType::Stud,
            "stud".to_string(),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(),
        );
        
        let coupling_b = Coupling::new_point(
            CouplingType::AntiStud,
            "antistud".to_string(),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(), // Same direction, not opposite
        );
        
        let transform_a = Matrix4::identity();
        let transform_b = Matrix4::identity();
        
        let result = find_connection_points(&coupling_a, &transform_a, &coupling_b, &transform_b);
        assert!(result.is_none());
    }

    #[test]
    fn test_calculate_connection_stress() {
        use crate::connection::{Connection, ConnectionState};
        use crate::coupling::{ConnectionMode, ConnectionProperties};
        
        let connection = Connection {
            id: Uuid::new_v4(),
            coupling_a: (Uuid::new_v4(), "a".to_string()),
            coupling_b: (Uuid::new_v4(), "b".to_string()),
            properties: ConnectionProperties {
                mode: ConnectionMode::Fixed,
                break_force: 10.0,
                break_torque: 5.0,
                dynamic: true,
                clutch_force: 8.0,
            },
            state: ConnectionState::default(),
            connection_point_a: Vector3::zero(),
            connection_point_b: Vector3::zero(),
        };
        
        // No stress
        let (force_stress, torque_stress) = calculate_connection_stress(
            &connection,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
        );
        assert_eq!(force_stress, 0.0);
        assert_eq!(torque_stress, 0.0);
        
        // Half stress
        let (force_stress, torque_stress) = calculate_connection_stress(
            &connection,
            Vector3::new(5.0, 0.0, 0.0),
            Vector3::new(0.0, 2.5, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
        );
        assert_eq!(force_stress, 0.5);
        assert_eq!(torque_stress, 0.5);
        
        // Over stress
        let (force_stress, torque_stress) = calculate_connection_stress(
            &connection,
            Vector3::new(20.0, 0.0, 0.0),
            Vector3::new(0.0, 10.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
        );
        assert_eq!(force_stress, 2.0);
        assert_eq!(torque_stress, 2.0);
    }

    #[test]
    fn test_couplings_within_range() {
        let coupling_a = Coupling::new_point(
            CouplingType::Stud,
            "stud".to_string(),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(),
        );
        
        let coupling_b = Coupling::new_point(
            CouplingType::AntiStud,
            "antistud".to_string(),
            Vector3::new(10.0, 0.0, 0.0),
            -Vector3::unit_y(),
        );
        
        let transform_a = Matrix4::identity();
        let transform_b = Matrix4::identity();
        
        assert!(couplings_within_range(&coupling_a, &transform_a, &coupling_b, &transform_b, 15.0));
        assert!(couplings_within_range(&coupling_a, &transform_a, &coupling_b, &transform_b, 10.0));
        assert!(!couplings_within_range(&coupling_a, &transform_a, &coupling_b, &transform_b, 5.0));
    }

    #[test]
    fn test_couplings_with_transforms() {
        let coupling_a = Coupling::new_point(
            CouplingType::Stud,
            "stud".to_string(),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(),
        );
        
        let coupling_b = Coupling::new_point(
            CouplingType::AntiStud,
            "antistud".to_string(),
            Vector3::new(0.0, 0.0, 0.0),
            -Vector3::unit_y(),
        );
        
        let transform_a = Matrix4::identity();
        let transform_b = Matrix4::from_translation(Vector3::new(5.0, 0.0, 0.0));
        
        assert!(couplings_within_range(&coupling_a, &transform_a, &coupling_b, &transform_b, 5.0));
        assert!(!couplings_within_range(&coupling_a, &transform_a, &coupling_b, &transform_b, 4.0));
    }

    #[test]
    fn test_project_point_to_linear_coupling() {
        let center = Vector3::new(0.0, 0.0, 0.0);
        let normal = Vector3::unit_x();
        let length = 10.0;
        
        // Point on line within bounds
        let point = Vector3::new(3.0, 0.0, 0.0);
        let projected = project_point_to_linear_coupling(point, center, normal, length);
        assert_eq!(projected, Vector3::new(3.0, 0.0, 0.0));
        
        // Point on line at boundary
        let point = Vector3::new(5.0, 0.0, 0.0);
        let projected = project_point_to_linear_coupling(point, center, normal, length);
        assert_eq!(projected, Vector3::new(5.0, 0.0, 0.0));
        
        // Point beyond boundary
        let point = Vector3::new(10.0, 0.0, 0.0);
        let projected = project_point_to_linear_coupling(point, center, normal, length);
        assert_eq!(projected, Vector3::new(5.0, 0.0, 0.0));
        
        // Point beyond negative boundary
        let point = Vector3::new(-10.0, 0.0, 0.0);
        let projected = project_point_to_linear_coupling(point, center, normal, length);
        assert_eq!(projected, Vector3::new(-5.0, 0.0, 0.0));
        
        // Point off line
        let point = Vector3::new(3.0, 2.0, 0.0);
        let projected = project_point_to_linear_coupling(point, center, normal, length);
        assert_eq!(projected, Vector3::new(3.0, 0.0, 0.0));
    }

    #[test]
    fn test_linear_geometry_connections() {
        let coupling_a = Coupling {
            coupling_type: CouplingType::FullPinHole,
            id: "pin_hole".to_string(),
            center: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::unit_x(),
            geometry: CouplingGeometry::Linear { length: 20.0 },
            excludes: Vec::new(),
        };
        
        let coupling_b = Coupling::new_point(
            CouplingType::Pin,
            "pin".to_string(),
            Vector3::new(10.0, 0.0, 0.0),
            -Vector3::unit_x(),
        );
        
        let transform_a = Matrix4::identity();
        let transform_b = Matrix4::identity();
        
        let result = find_connection_points(&coupling_a, &transform_a, &coupling_b, &transform_b);
        assert!(result.is_some());
    }
}
