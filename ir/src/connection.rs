use cgmath::{InnerSpace, SquareMatrix, Zero};
use ldraw::{Matrix4, Vector3};
use uuid::Uuid;

use crate::coupling::{CouplingType, ConnectionProperties, ConnectionMode};

/// Represents the current state of a connection during simulation
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionState {
    /// Current relative transform between the connected parts
    pub relative_transform: Matrix4,
    
    /// Current forces acting on the connection
    pub force: Vector3,
    
    /// Current torque acting on the connection
    pub torque: Vector3,
    
    /// For rotational connections, current angle
    pub angle: Option<f32>,
    
    /// For linear connections, current displacement
    pub displacement: Option<f32>,
    
    /// Whether the connection is currently intact
    pub intact: bool,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            relative_transform: Matrix4::identity(),
            force: Vector3::zero(),
            torque: Vector3::zero(),
            angle: None,
            displacement: None,
            intact: true,
        }
    }
}

/// A Connection represents a link between two Couplings on different parts
#[derive(Debug, Clone, PartialEq)]
pub struct Connection {
    /// Unique identifier for this connection
    pub id: Uuid,
    
    /// The coupling on the first part (part_id, coupling_id)
    pub coupling_a: (Uuid, String),
    
    /// The coupling on the second part (part_id, coupling_id)
    pub coupling_b: (Uuid, String),
    
    /// Physical properties of this connection
    pub properties: ConnectionProperties,
    
    /// Current state of the connection
    pub state: ConnectionState,
    
    /// Connection points in world space at time of connection
    pub connection_point_a: Vector3,
    pub connection_point_b: Vector3,
}

impl Connection {
    /// Create a new connection between two couplings
    pub fn new(
        coupling_a: (Uuid, String),
        coupling_b: (Uuid, String),
        type_a: CouplingType,
        type_b: CouplingType,
        connection_point_a: Vector3,
        connection_point_b: Vector3,
    ) -> Option<Self> {
        // Check if connection is valid
        let properties = type_a.can_connect_to(&type_b)?;
        
        Some(Self {
            id: Uuid::new_v4(),
            coupling_a,
            coupling_b,
            properties,
            state: ConnectionState::default(),
            connection_point_a,
            connection_point_b,
        })
    }
    
    /// Update connection state based on forces and check for breakage
    pub fn update(&mut self, _delta_time: f32) -> bool {
        // Check if forces exceed break limits
        if self.state.force.magnitude() > self.properties.break_force ||
           self.state.torque.magnitude() > self.properties.break_torque {
            self.state.intact = false;
            return false;
        }
        
        // Apply constraints based on connection mode
        match &self.properties.mode {
            ConnectionMode::Fixed => {
                // No movement allowed
            }
            ConnectionMode::Rotational { friction: _ } => {
                // Apply rotational friction
                // TODO: Implement physics simulation
            }
            ConnectionMode::Linear { friction: _ } => {
                // Apply linear friction
                // TODO: Implement physics simulation
            }
            ConnectionMode::RotationalLinear { .. } => {
                // Apply both frictions
                // TODO: Implement physics simulation
            }
            ConnectionMode::Hinge { min_angle, max_angle, friction: _ } => {
                // Constrain angle to limits
                if let Some(angle) = &mut self.state.angle {
                    *angle = angle.clamp(*min_angle, *max_angle);
                }
            }
            ConnectionMode::BallJoint { max_angle: _, friction: _ } => {
                // Constrain to cone of movement
                // TODO: Implement spherical constraints
            }
        }
        
        true
    }
}

/// Manages all connections in a model
#[derive(Debug, Clone, Default)]
pub struct ConnectionGraph {
    /// All active connections
    pub connections: Vec<Connection>,
    
    /// Index by part ID for quick lookup
    connections_by_part: std::collections::HashMap<Uuid, Vec<usize>>,
}

impl ConnectionGraph {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a new connection to the graph
    pub fn add_connection(&mut self, connection: Connection) {
        let idx = self.connections.len();
        
        // Update indices
        self.connections_by_part
            .entry(connection.coupling_a.0)
            .or_default()
            .push(idx);
        self.connections_by_part
            .entry(connection.coupling_b.0)
            .or_default()
            .push(idx);
            
        self.connections.push(connection);
    }
    
    /// Remove a connection
    pub fn remove_connection(&mut self, connection_id: Uuid) {
        self.connections.retain(|c| c.id != connection_id);
        self.rebuild_indices();
    }
    
    /// Get all connections for a specific part
    pub fn get_connections_for_part(&self, part_id: Uuid) -> Vec<&Connection> {
        self.connections_by_part
            .get(&part_id)
            .map(|indices| {
                indices.iter()
                    .filter_map(|&idx| self.connections.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Check if a coupling is already connected
    pub fn is_coupling_connected(&self, part_id: Uuid, coupling_id: &str) -> bool {
        self.get_connections_for_part(part_id)
            .iter()
            .any(|conn| {
                (conn.coupling_a.0 == part_id && conn.coupling_a.1 == coupling_id) ||
                (conn.coupling_b.0 == part_id && conn.coupling_b.1 == coupling_id)
            })
    }
    
    /// Update all connections and remove broken ones
    pub fn update(&mut self, delta_time: f32) {
        let mut broken_connections = Vec::new();
        
        for connection in &mut self.connections {
            if !connection.update(delta_time) {
                broken_connections.push(connection.id);
            }
        }
        
        // Remove broken connections
        for id in broken_connections {
            self.remove_connection(id);
        }
    }
    
    fn rebuild_indices(&mut self) {
        self.connections_by_part.clear();
        for (idx, connection) in self.connections.iter().enumerate() {
            self.connections_by_part
                .entry(connection.coupling_a.0)
                .or_default()
                .push(idx);
            self.connections_by_part
                .entry(connection.coupling_b.0)
                .or_default()
                .push(idx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coupling::CouplingType;

    #[test]
    fn test_connection_state_default() {
        let state = ConnectionState::default();
        assert_eq!(state.relative_transform, Matrix4::identity());
        assert_eq!(state.force, Vector3::zero());
        assert_eq!(state.torque, Vector3::zero());
        assert!(state.intact);
        assert_eq!(state.angle, None);
        assert_eq!(state.displacement, None);
    }

    #[test]
    fn test_connection_creation() {
        let part_a = Uuid::new_v4();
        let part_b = Uuid::new_v4();
        
        let connection = Connection::new(
            (part_a, "stud_0".to_string()),
            (part_b, "antistud_0".to_string()),
            CouplingType::Stud,
            CouplingType::AntiStud,
            Vector3::new(0.0, 10.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
        );
        
        assert!(connection.is_some());
        let connection = connection.unwrap();
        assert_eq!(connection.coupling_a.0, part_a);
        assert_eq!(connection.coupling_b.0, part_b);
        assert!(connection.state.intact);
    }

    #[test]
    fn test_incompatible_connection_creation() {
        let part_a = Uuid::new_v4();
        let part_b = Uuid::new_v4();
        
        let connection = Connection::new(
            (part_a, "stud_0".to_string()),
            (part_b, "stud_1".to_string()),
            CouplingType::Stud,
            CouplingType::Stud,
            Vector3::zero(),
            Vector3::zero(),
        );
        
        assert!(connection.is_none());
    }

    #[test]
    fn test_connection_break_on_force() {
        let part_a = Uuid::new_v4();
        let part_b = Uuid::new_v4();
        
        let mut connection = Connection::new(
            (part_a, "stud_0".to_string()),
            (part_b, "antistud_0".to_string()),
            CouplingType::Stud,
            CouplingType::AntiStud,
            Vector3::zero(),
            Vector3::zero(),
        ).unwrap();
        
        // Apply force below break limit
        connection.state.force = Vector3::new(5.0, 0.0, 0.0);
        assert!(connection.update(0.1));
        assert!(connection.state.intact);
        
        // Apply force above break limit
        connection.state.force = Vector3::new(15.0, 0.0, 0.0);
        assert!(!connection.update(0.1));
        assert!(!connection.state.intact);
    }

    #[test]
    fn test_connection_break_on_torque() {
        let part_a = Uuid::new_v4();
        let part_b = Uuid::new_v4();
        
        let mut connection = Connection::new(
            (part_a, "clip_0".to_string()),
            (part_b, "bar_0".to_string()),
            CouplingType::Clip,
            CouplingType::Bar,
            Vector3::zero(),
            Vector3::zero(),
        ).unwrap();
        
        // Apply torque above break limit
        connection.state.torque = Vector3::new(0.0, 5.0, 0.0);
        assert!(!connection.update(0.1));
        assert!(!connection.state.intact);
    }

    #[test]
    fn test_connection_graph_add_remove() {
        let mut graph = ConnectionGraph::new();
        let part_a = Uuid::new_v4();
        let part_b = Uuid::new_v4();
        
        let connection = Connection::new(
            (part_a, "stud_0".to_string()),
            (part_b, "antistud_0".to_string()),
            CouplingType::Stud,
            CouplingType::AntiStud,
            Vector3::zero(),
            Vector3::zero(),
        ).unwrap();
        
        let connection_id = connection.id;
        graph.add_connection(connection);
        
        assert_eq!(graph.connections.len(), 1);
        assert_eq!(graph.get_connections_for_part(part_a).len(), 1);
        assert_eq!(graph.get_connections_for_part(part_b).len(), 1);
        
        graph.remove_connection(connection_id);
        assert_eq!(graph.connections.len(), 0);
        assert_eq!(graph.get_connections_for_part(part_a).len(), 0);
    }

    #[test]
    fn test_is_coupling_connected() {
        let mut graph = ConnectionGraph::new();
        let part_a = Uuid::new_v4();
        let part_b = Uuid::new_v4();
        
        let connection = Connection::new(
            (part_a, "stud_0".to_string()),
            (part_b, "antistud_0".to_string()),
            CouplingType::Stud,
            CouplingType::AntiStud,
            Vector3::zero(),
            Vector3::zero(),
        ).unwrap();
        
        graph.add_connection(connection);
        
        assert!(graph.is_coupling_connected(part_a, "stud_0"));
        assert!(graph.is_coupling_connected(part_b, "antistud_0"));
        assert!(!graph.is_coupling_connected(part_a, "stud_1"));
    }

    #[test]
    fn test_connection_graph_update() {
        let mut graph = ConnectionGraph::new();
        let part_a = Uuid::new_v4();
        let part_b = Uuid::new_v4();
        
        let mut connection = Connection::new(
            (part_a, "stud_0".to_string()),
            (part_b, "antistud_0".to_string()),
            CouplingType::Stud,
            CouplingType::AntiStud,
            Vector3::zero(),
            Vector3::zero(),
        ).unwrap();
        
        // Set force above break limit
        connection.state.force = Vector3::new(20.0, 0.0, 0.0);
        graph.add_connection(connection);
        
        assert_eq!(graph.connections.len(), 1);
        graph.update(0.1);
        assert_eq!(graph.connections.len(), 0); // Broken connection removed
    }
}
