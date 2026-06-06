use verium_pool_miner::Scratchpad;

/// Golden digest for empty 80-byte header, N=1048576 (veriumMiner `tests/golden.h`).
const GOLDEN_EMPTY: &str = "ef9dc4d21dc073154996e0fbdebc4c014d6defa4a3791ab14bc5def68b0d76dc";

#[test]
fn empty_header_matches_golden() {
    let header = [0u8; 80];
    let scratch = Scratchpad::new();
    let hash = scratch.hash(&header);
    let got = hex::encode(hash);
    assert_eq!(got, GOLDEN_EMPTY, "scrypt2 consensus hash mismatch");
}
