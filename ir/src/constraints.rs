use ldraw::Vector3;
use uuid::Uuid;

use crate::connection::ConnectionGraph;
use crate::coupling::ConnectionMode;

/// Constraint types for physics simulation
#[derive(Debug, Clone)]
pub enum Constraint {
    /// Keep two points at the same position
    Point {
        part_a: Uuid,
        part_b: Uuid,
        local_point_a: Vector3,
        local_point_b: Vector3,
    },
    /// Keep two axes aligned (for hinges)
    Hinge {
        part_a: Uuid,
        part_b: Uuid,
        local_axis_a: Vector3,
        local_axis_b: Vector3,
        min_angle: f32,
        max_angle: f32,
    },
    /// Ball joint constraint
    Ball {
        part_a: Uuid,
        part_b: Uuid,
        local_point_a: Vector3,
        local_point_b: Vector3,
        max_angle: f32,
    },
    /// Linear constraint (sliding)
    Linear {
        part_a: Uuid,
        part_b: Uuid,
        local_axis: Vector3,
        min_displacement: f32,
        max_displacement: f32,
    },
}

/// Solver for maintaining connection constraints
pub struct ConstraintSolver {
    constraints: Vec<Constraint>,
}

impl ConstraintSolver {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }
    
    /// Build constraints from connection graph
    pub fn build_constraints(&mut self, connections: &ConnectionGraph) {
        self.constraints.clear();
        
        for connection in &connections.connections {
            if !connection.state.intact {
                continue;
            }
            
            match &connection.properties.mode {
                ConnectionMode::Fixed => {
                    self.constraints.push(Constraint::Point {
                        part_a: connection.coupling_a.0,
                        part_b: connection.coupling_b.0,
                        local_point_a: connection.connection_point_a,
                        local_point_b: connection.connection_point_b,
                    });
                }
                ConnectionMode::Hinge { min_angle, max_angle, .. } => {
                    self.constraints.push(Constraint::Hinge {
                        part_a: connection.coupling_a.0,
                        part_b: connection.coupling_b.0,
                        local_axis_a: Vector3::unit_z(), // TODO: Get from coupling normal
                        local_axis_b: Vector3::unit_z(),
                        min_angle: *min_angle,
                        max_angle: *max_angle,
                    });
                }
                ConnectionMode::BallJoint { max_angle, .. } => {
                    self.constraints.push(Constraint::Ball {
                        part_a: connection.coupling_a.0,
                        part_b: connection.coupling_b.0,
                        local_point_a: connection.connection_point_a,
                        local_point_b: connection.connection_point_b,
                        max_angle: *max_angle,
                    });
                }
                ConnectionMode::Linear { .. } => {
                    self.constraints.push(Constraint::Linear {
                        part_a: connection.coupling_a.0,
                        part_b: connection.coupling_b.0,
                        local_axis: Vector3::unit_z(), // TODO: Get from coupling normal
                        min_displacement: -10.0, // TODO: Get from geometry
                        max_displacement: 10.0,
                    });
                }
                _ => {
                    // TODO: Handle other connection modes
                }
            }
        }
    }
    
    /// Get all constraints
    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }
    
    /// Solve constraints (placeholder - would need Instance type from model module)
    pub fn solve(&self, _iterations: usize) {
        // TODO: Implement constraint solving when Instance type is available
        // This would adjust transforms to maintain constraints
        // for _ in 0..iterations {
        //     for constraint in &self.constraints {
        //         match constraint {
        //             Constraint::Point { .. } => {
        //                 // Adjust transforms to keep points coincident
        //             }
        //             Constraint::Hinge { .. } => {
        //                 // Maintain hinge axis alignment and angle limits
        //             }
        //             Constraint::Ball { .. } => {
        //                 // Maintain ball joint constraints
        //             }
        //             Constraint::Linear { .. } => {
        //                 // Maintain linear sliding constraints
        //             }
        //         }
        //     }
        // }
    }
    
    /// Apply impulse to satisfy a constraint
    pub fn apply_constraint_impulse(&self, constraint: &Constraint, _dt: f32) {
        match constraint {
            Constraint::Point { .. } => {
                // Calculate and apply impulse to bring points together
            }
            Constraint::Hinge { .. } => {
                // Calculate and apply impulse to maintain hinge constraint
            }
            Constraint::Ball { .. } => {
                // Calculate and apply impulse for ball joint
            }
            Constraint::Linear { .. } => {
                // Calculate and apply impulse for linear constraint
            }
        }
    }
}

impl Default for ConstraintSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::{Connection, ConnectionGraph, ConnectionState};
    use crate::coupling::{ConnectionMode, ConnectionProperties};

    fn create_test_connection(
        part_a: Uuid,
        part_b: Uuid,
        mode: ConnectionMode,
    ) -> Connection {
        Connection {
            id: Uuid::new_v4(),
            coupling_a: (part_a, "coupling_a".to_string()),
            coupling_b: (part_b, "coupling_b".to_string()),
            properties: ConnectionProperties {
                mode,
                break_force: 10.0,
                break_torque: 5.0,
                dynamic: true,
            },
            state: ConnectionState::default(),
            connection_point_a: Vector3::new(0.0, 0.0, 0.0),
            connection_point_b: Vector3::new(10.0, 0.0, 0.0),
        }
    }

    #[test]
    fn test_constraint_solver_creation() {
        let solver = ConstraintSolver::new();
        assert_eq!(solver.constraints().len(), 0);
    }

    #[test]
    fn test_build_point_constraints() {
        let mut solver = ConstraintSolver::new();
        let mut graph = ConnectionGraph::new();
        
        let part_a = Uuid::new_v4();
        let part_b = Uuid::new_v4();
        
        let connection = create_test_connection(part_a, part_b, ConnectionMode::Fixed);
        graph.add_connection(connection);
        
        solver.build_constraints(&graph);
        assert_eq!(solver.constraints().len(), 1);
        
        match &solver.constraints()[0] {
            Constraint::Point { part_a: pa, part_b: pb, .. } => {
                assert_eq!(*pa, part_a);
                assert_eq!(*pb, part_b);
            }
            _ => panic!("Expected Point constraint"),
        }
    }

    #[test]
    fn test_build_hinge_constraints() {
        let mut solver = ConstraintSolver::new();
        let mut graph = ConnectionGraph::new();
        
        let part_a = Uuid::new_v4();
        let part_b = Uuid::new_v4();
        
        let connection = create_test_connection(
            part_a,
            part_b,
            ConnectionMode::Hinge {
                min_angle: -90.0,
                max_angle: 90.0,
                friction: 0.5,
            },
        );
        graph.add_connection(connection);
        
        solver.build_constraints(&graph);
        assert_eq!(solver.constraints().len(), 1);
        
        match &solver.constraints()[0] {
            Constraint::Hinge { min_angle, max_angle, .. } => {
                assert_eq!(*min_angle, -90.0);
                assert_eq!(*max_angle, 90.0);
            }
            _ => panic!("Expected Hinge constraint"),
        }
    }

    #[test]
    fn test_skip_broken_connections() {
        let mut solver = ConstraintSolver::new();
        let mut graph = ConnectionGraph::new();
        
        let part_a = Uuid::new_v4();
        let part_b = Uuid::new_v4();
        
        let mut connection = create_test_connection(part_a, part_b, ConnectionMode::Fixed);
        connection.state.intact = false; // Mark as broken
        graph.add_connection(connection);
        
        solver.build_constraints(&graph);
        assert_eq!(solver.constraints().len(), 0); // No constraints for broken connections
    }

    #[test]
    fn test_ball_joint_constraints() {
        let mut solver = ConstraintSolver::new();
        let mut graph = ConnectionGraph::new();
        
        let part_a = Uuid::new_v4();
        let part_b = Uuid::new_v4();
        
        let connection = create_test_connection(
            part_a,
            part_b,
            ConnectionMode::BallJoint {
                max_angle: 45.0,
                friction: 0.3,
            },
        );
        graph.add_connection(connection);
        
        solver.build_constraints(&graph);
        assert_eq!(solver.constraints().len(), 1);
        
        match &solver.constraints()[0] {
            Constraint::Ball { max_angle, .. } => {
                assert_eq!(*max_angle, 45.0);
            }
            _ => panic!("Expected Ball constraint"),
        }
    }

    #[test]
    fn test_linear_constraints() {
        let mut solver = ConstraintSolver::new();
        let mut graph = ConnectionGraph::new();
        
        let part_a = Uuid::new_v4();
        let part_b = Uuid::new_v4();
        
        let connection = create_test_connection(
            part_a,
            part_b,
            ConnectionMode::Linear { friction: 0.2 },
        );
        graph.add_connection(connection);
        
        solver.build_constraints(&graph);
        assert_eq!(solver.constraints().len(), 1);
        
        match &solver.constraints()[0] {
            Constraint::Linear { min_displacement, max_displacement, .. } => {
                assert_eq!(*min_displacement, -10.0);
                assert_eq!(*max_displacement, 10.0);
            }
            _ => panic!("Expected Linear constraint"),
        }
    }

    #[test]
    fn test_multiple_constraints() {
        let mut solver = ConstraintSolver::new();
        let mut graph = ConnectionGraph::new();
        
        let part_a = Uuid::new_v4();
        let part_b = Uuid::new_v4();
        let part_c = Uuid::new_v4();
        
        // Add multiple connections
        graph.add_connection(create_test_connection(part_a, part_b, ConnectionMode::Fixed));
        graph.add_connection(create_test_connection(
            part_b,
            part_c,
            ConnectionMode::Hinge {
                min_angle: -180.0,
                max_angle: 180.0,
                friction: 0.1,
            },
        ));
        
        solver.build_constraints(&graph);
        assert_eq!(solver.constraints().len(), 2);
    }
}