#[cfg(test)]
mod tests {
    use crate::cube::{Cube, Move};
    use crate::solver::coord::*;

    #[test]
    fn test_move_cubes_basic() {
        for m_idx in 0..6 {
            let rc_move = RawCube::move_cube(m_idx);
            // Move should not be empty
            assert_ne!(rc_move.cp, RawCube::default().cp);
        }
    }

    #[test]
    fn test_move_identity() {
        let mut rc = RawCube::default();
        // apply R, then Rp
        rc = rc.multiply(RawCube::move_cube(1)); // R
        let r2 = RawCube::move_cube(1).multiply(RawCube::move_cube(1));
        rc = rc.multiply(&r2); // R2
        rc = rc.multiply(RawCube::move_cube(1)); // R (total R4 = Identity)
        assert_eq!(rc.cp, RawCube::default().cp);
        assert_eq!(rc.co, RawCube::default().co);
    }

    #[test]
    fn test_from_cube_conversion() {
        let cube = Cube::new();
        let rc = RawCube::from_cube(&cube, &[0, 1, 2, 3, 4, 5]).unwrap();
        assert_eq!(rc.cp, RawCube::default().cp);
        assert_eq!(rc.co, RawCube::default().co);
    }

    #[test]
    fn test_move_table_consistency_detailed() {
        use crate::solver::tables::MoveTable;
        let mt = MoveTable::get();
        let mut cube = Cube::new();
        cube.apply_move(Move::U);
        let rc = RawCube::from_cube(&cube, &[0, 1, 2, 3, 4, 5]).unwrap();
        let cp = rc.get_cp();
        println!("CP after U: {}", cp);
        
        // m=2 is Up
        let next_cp = mt.cp[cp as usize][2];
        println!("CP after U then Up: {}", next_cp);
        assert_eq!(next_cp, 0, "U then Up should lead to CP 0");
    }

    #[test]
    fn test_all_moves_consistency() {
        let moves = [Move::U, Move::D, Move::L, Move::R, Move::F, Move::B];

        for (i, &mv) in moves.iter().enumerate() {
            let mut cube = Cube::new();
            cube.apply_move(mv);

            let rc_from_cube = RawCube::from_cube(&cube, &[0, 1, 2, 3, 4, 5]).unwrap();

            let mut rc_simulated = RawCube::default();
            rc_simulated = rc_simulated.multiply(RawCube::move_cube(i));

            assert_eq!(
                rc_from_cube, rc_simulated,
                "Move {:?} (idx {}) consistency failed",
                mv, i
            );
        }
    }
}
