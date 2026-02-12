use rubiks_cube_2x2::solver::search::Search;

#[test]
fn test_search_extra_coverage() {
    // Transferred from src/solver/search.rs
    let search = Search::default();
    assert_eq!(search.max_nodes, 10_000_000);
}
