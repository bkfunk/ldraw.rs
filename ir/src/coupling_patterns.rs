use cgmath::InnerSpace;
use ldraw::Vector3;

use crate::coupling::{Coupling, CouplingGeometry, CouplingType};

/// Common coupling patterns for LEGO parts
pub struct CouplingPatterns;

impl CouplingPatterns {
    /// Standard stud spacing in LDraw units (20 LDU = 8mm)
    pub const STUD_SPACING: f32 = 20.0;

    /// Standard plate height in LDraw units
    pub const PLATE_HEIGHT: f32 = 8.0;

    /// Standard brick height in LDraw units
    pub const BRICK_HEIGHT: f32 = 24.0;

    /// Create studs for a rectangular brick/plate
    pub fn create_studs(width: u32, length: u32) -> Vec<Coupling> {
        let mut couplings = Vec::new();

        for x in 0..width {
            for z in 0..length {
                let center = Vector3::new(
                    (x as f32 - (width - 1) as f32 / 2.0) * Self::STUD_SPACING,
                    0.0,
                    (z as f32 - (length - 1) as f32 / 2.0) * Self::STUD_SPACING,
                );

                couplings.push(Coupling::new_point(
                    CouplingType::Stud,
                    format!("stud_{x}_{z}"),
                    center,
                    Vector3::unit_y(),
                ));
            }
        }

        couplings
    }

    /// Create anti-studs (bottom connections) for a rectangular brick/plate
    pub fn create_antistuds(width: u32, length: u32, height_type: HeightType) -> Vec<Coupling> {
        let mut couplings = Vec::new();
        let y_offset = match height_type {
            HeightType::Plate => -Self::PLATE_HEIGHT,
            HeightType::Brick => -Self::BRICK_HEIGHT,
        };

        // Corner anti-studs
        let x_positions: Vec<u32> = if width == 1 {
            vec![0]
        } else {
            vec![0, width - 1]
        };
        let z_positions: Vec<u32> = if length == 1 {
            vec![0]
        } else {
            vec![0, length - 1]
        };

        for &x in &x_positions {
            for &z in &z_positions {
                let center = Vector3::new(
                    (x as f32 - (width - 1) as f32 / 2.0) * Self::STUD_SPACING,
                    y_offset,
                    (z as f32 - (length - 1) as f32 / 2.0) * Self::STUD_SPACING,
                );

                couplings.push(Coupling::new_point(
                    CouplingType::AntiStud,
                    format!("antistud_{x}_{z}"),
                    center,
                    -Vector3::unit_y(),
                ));
            }
        }

        // Add tubes for larger parts
        if width >= 2 && length >= 2 {
            for x in 0..width - 1 {
                for z in 0..length - 1 {
                    let center = Vector3::new(
                        (x as f32 - (width - 2) as f32 / 2.0) * Self::STUD_SPACING,
                        y_offset,
                        (z as f32 - (length - 2) as f32 / 2.0) * Self::STUD_SPACING,
                    );

                    couplings.push(Coupling {
                        coupling_type: CouplingType::Tube,
                        id: format!("tube_{x}_{z}"),
                        center,
                        normal: -Vector3::unit_y(),
                        geometry: CouplingGeometry::Circular { radius: 6.0 },
                        excludes: Vec::new(),
                    });
                }
            }
        }

        couplings
    }

    /// Create a technic pin hole
    pub fn create_pin_hole(id: String, center: Vector3, axis: Vector3) -> Coupling {
        Coupling {
            coupling_type: CouplingType::FullPinHole,
            id,
            center,
            normal: axis.normalize(),
            geometry: CouplingGeometry::Linear { length: 20.0 },
            excludes: Vec::new(),
        }
    }

    /// Create a technic axle hole
    pub fn create_axle_hole(id: String, center: Vector3, axis: Vector3) -> Coupling {
        Coupling {
            coupling_type: CouplingType::AxleHole,
            id,
            center,
            normal: axis.normalize(),
            geometry: CouplingGeometry::Linear { length: 20.0 },
            excludes: Vec::new(),
        }
    }

    /// Create clip couplings (mutually exclusive)
    pub fn create_clip(id_base: &str, center: Vector3, direction: Vector3) -> Vec<Coupling> {
        vec![
            Coupling {
                coupling_type: CouplingType::Clip,
                id: format!("{id_base}_closed"),
                center,
                normal: direction.normalize(),
                geometry: CouplingGeometry::Point,
                excludes: vec![format!("{}_open", id_base)],
            },
            Coupling {
                coupling_type: CouplingType::ClipOpen,
                id: format!("{id_base}_open"),
                center,
                normal: direction.normalize(),
                geometry: CouplingGeometry::Point,
                excludes: vec![format!("{}_closed", id_base)],
            },
        ]
    }

    /// Create standard hinge couplings
    pub fn create_hinge(id: &str, center: Vector3, axis: Vector3, is_male: bool) -> Coupling {
        Coupling {
            coupling_type: if is_male {
                CouplingType::Hinge1x2BrickMale
            } else {
                CouplingType::Hinge1x2BrickFemale
            },
            id: id.to_string(),
            center,
            normal: axis.normalize(),
            geometry: CouplingGeometry::Point,
            excludes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HeightType {
    Plate,
    Brick,
}

/// Helper to generate couplings for common part types
pub fn generate_brick_couplings(width: u32, length: u32, height_type: HeightType) -> Vec<Coupling> {
    let mut couplings = Vec::new();

    // Add top studs
    couplings.extend(CouplingPatterns::create_studs(width, length));

    // Add bottom connections
    couplings.extend(CouplingPatterns::create_antistuds(
        width,
        length,
        height_type,
    ));

    couplings
}

/// Helper to generate couplings for a technic beam
pub fn generate_technic_beam_couplings(length: u32) -> Vec<Coupling> {
    let mut couplings = Vec::new();

    for i in 0..length {
        let x = (i as f32 - (length - 1) as f32 / 2.0) * CouplingPatterns::STUD_SPACING;

        // Add pin holes along the beam
        couplings.push(CouplingPatterns::create_pin_hole(
            format!("pin_hole_{i}"),
            Vector3::new(x, 0.0, 0.0),
            Vector3::unit_z(),
        ));
    }

    couplings
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::assert_abs_diff_eq;

    #[test]
    fn test_create_studs_single() {
        let studs = CouplingPatterns::create_studs(1, 1);
        assert_eq!(studs.len(), 1);
        assert_eq!(studs[0].coupling_type, CouplingType::Stud);
        assert_eq!(studs[0].center, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(studs[0].normal, Vector3::unit_y());
        assert_eq!(studs[0].id, "stud_0_0");
    }

    #[test]
    fn test_create_studs_2x2() {
        let studs = CouplingPatterns::create_studs(2, 2);
        assert_eq!(studs.len(), 4);

        // Check corners are positioned correctly
        let expected_positions = vec![
            Vector3::new(-10.0, 0.0, -10.0), // stud_0_0
            Vector3::new(-10.0, 0.0, 10.0),  // stud_0_1
            Vector3::new(10.0, 0.0, -10.0),  // stud_1_0
            Vector3::new(10.0, 0.0, 10.0),   // stud_1_1
        ];

        for stud in &studs {
            assert!(expected_positions.iter().any(|&pos| stud.center == pos));
        }
    }

    #[test]
    fn test_create_antistuds_2x2_plate() {
        let antistuds = CouplingPatterns::create_antistuds(2, 2, HeightType::Plate);

        // 2x2 has 4 corner antistuds + 1 center tube
        assert_eq!(antistuds.len(), 5);

        // Check antistuds
        let antistud_count = antistuds
            .iter()
            .filter(|c| c.coupling_type == CouplingType::AntiStud)
            .count();
        assert_eq!(antistud_count, 4);

        // Check tube
        let tube_count = antistuds
            .iter()
            .filter(|c| c.coupling_type == CouplingType::Tube)
            .count();
        assert_eq!(tube_count, 1);

        // Check height
        for coupling in &antistuds {
            assert_eq!(coupling.center.y, -8.0); // Plate height
            assert_eq!(coupling.normal, -Vector3::unit_y());
        }
    }

    #[test]
    fn test_create_antistuds_1x1_brick() {
        let antistuds = CouplingPatterns::create_antistuds(1, 1, HeightType::Brick);

        // 1x1 has only 1 corner antistud, no tubes
        assert_eq!(antistuds.len(), 1);
        assert_eq!(antistuds[0].coupling_type, CouplingType::AntiStud);
        assert_eq!(antistuds[0].center, Vector3::new(0.0, -24.0, 0.0));
    }

    #[test]
    fn test_create_antistuds_3x3() {
        let antistuds = CouplingPatterns::create_antistuds(3, 3, HeightType::Plate);

        // 3x3 has 4 corner antistuds + 4 tubes (2x2 grid)
        let antistud_count = antistuds
            .iter()
            .filter(|c| c.coupling_type == CouplingType::AntiStud)
            .count();
        assert_eq!(antistud_count, 4);

        let tube_count = antistuds
            .iter()
            .filter(|c| c.coupling_type == CouplingType::Tube)
            .count();
        assert_eq!(tube_count, 4);
    }

    #[test]
    fn test_create_pin_hole() {
        let pin_hole = CouplingPatterns::create_pin_hole(
            "test_pin".to_string(),
            Vector3::new(10.0, 5.0, 0.0),
            Vector3::new(1.0, 1.0, 0.0), // Non-normalized
        );

        assert_eq!(pin_hole.coupling_type, CouplingType::FullPinHole);
        assert_eq!(pin_hole.id, "test_pin");
        assert_eq!(pin_hole.center, Vector3::new(10.0, 5.0, 0.0));

        // Check normal is normalized
        assert_abs_diff_eq!(pin_hole.normal.magnitude(), 1.0, epsilon = 0.001);

        // Check geometry
        match pin_hole.geometry {
            CouplingGeometry::Linear { length } => assert_eq!(length, 20.0),
            _ => panic!("Expected Linear geometry"),
        }
    }

    #[test]
    fn test_create_axle_hole() {
        let axle_hole = CouplingPatterns::create_axle_hole(
            "test_axle".to_string(),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::unit_z(),
        );

        assert_eq!(axle_hole.coupling_type, CouplingType::AxleHole);
        assert_eq!(axle_hole.id, "test_axle");
        match axle_hole.geometry {
            CouplingGeometry::Linear { length } => assert_eq!(length, 20.0),
            _ => panic!("Expected Linear geometry"),
        }
    }

    #[test]
    fn test_create_clip() {
        let clips = CouplingPatterns::create_clip(
            "test_clip",
            Vector3::new(0.0, 0.0, 5.0),
            Vector3::unit_x(),
        );

        assert_eq!(clips.len(), 2);

        // Check closed clip
        let closed_clip = &clips[0];
        assert_eq!(closed_clip.coupling_type, CouplingType::Clip);
        assert_eq!(closed_clip.id, "test_clip_closed");
        assert_eq!(closed_clip.excludes, vec!["test_clip_open"]);

        // Check open clip
        let open_clip = &clips[1];
        assert_eq!(open_clip.coupling_type, CouplingType::ClipOpen);
        assert_eq!(open_clip.id, "test_clip_open");
        assert_eq!(open_clip.excludes, vec!["test_clip_closed"]);
    }

    #[test]
    fn test_create_hinge() {
        let male_hinge = CouplingPatterns::create_hinge(
            "hinge_m",
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::unit_x(),
            true,
        );

        assert_eq!(male_hinge.coupling_type, CouplingType::Hinge1x2BrickMale);
        assert_eq!(male_hinge.id, "hinge_m");

        let female_hinge = CouplingPatterns::create_hinge(
            "hinge_f",
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::unit_x(),
            false,
        );

        assert_eq!(female_hinge.coupling_type, CouplingType::Hinge1x2BrickFemale);
    }

    #[test]
    fn test_generate_brick_couplings() {
        let couplings = generate_brick_couplings(2, 3, HeightType::Brick);

        // 2x3 brick has 6 studs on top
        let stud_count = couplings
            .iter()
            .filter(|c| c.coupling_type == CouplingType::Stud)
            .count();
        assert_eq!(stud_count, 6);

        // Check that we have antistuds and tubes
        let antistud_count = couplings
            .iter()
            .filter(|c| c.coupling_type == CouplingType::AntiStud)
            .count();
        assert!(antistud_count > 0);
    }

    #[test]
    fn test_generate_technic_beam_couplings() {
        let couplings = generate_technic_beam_couplings(5);

        assert_eq!(couplings.len(), 5);

        for (i, coupling) in couplings.iter().enumerate() {
            assert_eq!(coupling.coupling_type, CouplingType::FullPinHole);
            assert_eq!(coupling.id, format!("pin_hole_{}", i));
            assert_eq!(coupling.normal, Vector3::unit_z());

            // Check spacing
            let expected_x = (i as f32 - 2.0) * 20.0;
            assert_eq!(coupling.center.x, expected_x);
        }
    }

    #[test]
    fn test_constants() {
        assert_eq!(CouplingPatterns::STUD_SPACING, 20.0);
        assert_eq!(CouplingPatterns::PLATE_HEIGHT, 8.0);
        assert_eq!(CouplingPatterns::BRICK_HEIGHT, 24.0);
    }
}
