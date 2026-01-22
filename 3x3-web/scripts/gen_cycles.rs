#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Face {
    Up,
    Down,
    Left,
    Right,
    Front,
    Back,
}

fn get_id(f: Face, row: i32, col: i32) -> usize {
    let base = match f {
        Face::Up => 0,
        Face::Down => 9,
        Face::Left => 18,
        Face::Right => 27,
        Face::Front => 36,
        Face::Back => 45,
    };
    base + (row * 3 + col) as usize
}

fn get_coords(f: Face, r: i32, c: i32) -> (f32, f32, f32) {
    let rs = (r as f32 - 1.0) * -1.0;
    let cs = c as f32 - 1.0;
    match f {
        Face::Up => (cs, 1.5, -rs),
        Face::Down => (cs, -1.5, rs),
        Face::Left => (-1.5, rs, cs),
        Face::Right => (1.5, rs, -cs),
        Face::Front => (cs, rs, 1.5),
        Face::Back => (-cs, rs, -1.5),
    }
}

fn find_id(x: f32, y: f32, z: f32) -> usize {
    let faces = [
        Face::Up,
        Face::Down,
        Face::Left,
        Face::Right,
        Face::Front,
        Face::Back,
    ];
    for f in faces {
        for r in 0..3 {
            for c in 0..3 {
                let (sx, sy, sz) = get_coords(f, r, c);
                if (x - sx).abs() < 0.1 && (y - sy).abs() < 0.1 && (z - sz).abs() < 0.1 {
                    return get_id(f, r, c);
                }
            }
        }
    }
    panic!("No sticker at ({}, {}, {})", x, y, z);
}

fn main() {
    let corner_coords = [
        ("UFR", 1.0, 1.0, 1.0),
        ("UFL", -1.0, 1.0, 1.0),
        ("ULB", -1.0, 1.0, -1.0),
        ("UBR", 1.0, 1.0, -1.0),
        ("DFR", 1.0, -1.0, 1.0),
        ("DLF", -1.0, -1.0, 1.0),
        ("DBL", -1.0, -1.0, -1.0),
        ("DRB", 1.0, -1.0, -1.0),
    ];

    for (name, cx, cy, cz) in corner_coords {
        let mut ids = Vec::new();
        // Each corner has 3 stickers. They are at distances 1.5 from center.
        // For UFR (1,1,1), the stickers are at (1,1,1.5) on Front, (1.5,1,1) on Right, (1,1.5,1) on Up.
        ids.push(find_id(cx, 1.5, cz)); // Up/Down sticker

        // Next two depend on the corner.
        // For UFR, it's Front and Right.
        // Let's just find all 3 stickers near this corner.
        let mut stickers = Vec::new();
        for f in [
            Face::Up,
            Face::Down,
            Face::Left,
            Face::Right,
            Face::Front,
            Face::Back,
        ] {
            for r in 0..3 {
                for c in 0..3 {
                    let (sx, sy, sz) = get_coords(f, r, c);
                    if (cx - sx).abs() < 0.6 && (cy - sy).abs() < 0.6 && (cz - sz).abs() < 0.6 {
                        stickers.push(get_id(f, r, c));
                    }
                }
            }
        }
        println!("{}: {:?}", name, stickers);
    }

    println!("\nEdges:");
    let edge_coords = [
        ("UR", 1.0, 1.0, 0.0),
        ("UF", 0.0, 1.0, 1.0),
        ("UL", -1.0, 1.0, 0.0),
        ("UB", 0.0, 1.0, -1.0),
        ("DR", 1.0, -1.0, 0.0),
        ("DF", 0.0, -1.0, 1.0),
        ("DL", -1.0, -1.0, 0.0),
        ("DB", 0.0, -1.0, -1.0),
        ("FR", 1.0, 0.0, 1.0),
        ("FL", -1.0, 0.0, 1.0),
        ("BL", -1.0, 0.0, -1.0),
        ("BR", 1.0, 0.0, -1.0),
    ];
    for (name, cx, cy, cz) in edge_coords {
        let mut stickers = Vec::new();
        for f in [
            Face::Up,
            Face::Down,
            Face::Left,
            Face::Right,
            Face::Front,
            Face::Back,
        ] {
            for r in 0..3 {
                for c in 0..3 {
                    let (sx, sy, sz) = get_coords(f, r, c);
                    if (cx - sx).abs() < 0.6 && (cy - sy).abs() < 0.6 && (cz - sz).abs() < 0.6 {
                        stickers.push(get_id(f, r, c));
                    }
                }
            }
        }
        println!("{}: {:?}", name, stickers);
    }
}
