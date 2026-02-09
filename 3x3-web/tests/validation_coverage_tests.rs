use rubiks_cube_3x3::cube::{Color, Cube};

/// Test validation error: corner with opposite face colors
#[test]
fn test_validation_opposite_colors_in_corner() {
    let mut colors = [Color::White; 54];

    // Set up valid cube structure first
    for (i, face_color) in [
        Color::White,
        Color::Yellow,
        Color::Orange,
        Color::Red,
        Color::Green,
        Color::Blue,
    ]
    .iter()
    .enumerate()
    {
        for j in 0..9 {
            colors[i * 9 + j] = *face_color;
        }
    }

    // Create invalid corner: White and Yellow (opposite faces) in same corner
    colors[0] = Color::White; // U0
    colors[27] = Color::Yellow; // L0 (should form a corner with U0)
    colors[36] = Color::Green; // F0

    let result = Cube::from_colors(&colors);
    assert!(
        result.is_err(),
        "Should fail with invalid color configuration"
    );
}

/// Test validation error: duplicate corner pieces
#[test]
fn test_validation_duplicate_corners() {
    let mut colors = [Color::White; 54];

    // Set up basic structure
    for (i, face_color) in [
        Color::White,
        Color::Yellow,
        Color::Orange,
        Color::Red,
        Color::Green,
        Color::Blue,
    ]
    .iter()
    .enumerate()
    {
        for j in 0..9 {
            colors[i * 9 + j] = *face_color;
        }
    }

    // Create duplicate corners by setting same color pattern
    // This will likely be caught during validation
    colors[0] = Color::White;
    colors[2] = Color::White;

    let result = Cube::from_colors(&colors);
    // Should fail validation due to duplicate or invalid configuration
    assert!(result.is_err());
}

/// Test validation error: corner without white or yellow
#[test]
fn test_validation_corner_missing_white_yellow() {
    let mut colors = [Color::White; 54];

    // Set up structure where a corner doesn't have white or yellow
    for (i, face_color) in [
        Color::White,
        Color::Yellow,
        Color::Orange,
        Color::Red,
        Color::Green,
        Color::Blue,
    ]
    .iter()
    .enumerate()
    {
        for j in 0..9 {
            colors[i * 9 + j] = *face_color;
        }
    }

    // Make a corner that has only side colors (no White/Yellow)
    colors[0] = Color::Orange; // Should have White
    colors[27] = Color::Red;
    colors[36] = Color::Green;

    let result = Cube::from_colors(&colors);
    assert!(result.is_err());
}
