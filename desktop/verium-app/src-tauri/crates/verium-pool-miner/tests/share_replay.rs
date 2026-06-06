use verium_pool_miner::{reconstruct_share, Scratchpad, ShareSubmission, StratumJob};

const FIXTURE: &str =
    include_str!("../../../../../../../verium-pool/packages/protocol/test/fixtures/share-replay.json");

#[test]
fn share_replay_fixture_matches_golden() {
    let v: serde_json::Value = serde_json::from_str(FIXTURE).expect("parse fixture");
    let job = v.get("job").expect("job");
    let submit = v.get("submit").expect("submit");
    let expected = v.get("expected").expect("expected");

    let stratum_job = StratumJob {
        job_id: job["jobId"].as_str().unwrap().into(),
        prev_hash_hex: job["prevHashHex"].as_str().unwrap().into(),
        coinb1_hex: job["coinb1Hex"].as_str().unwrap().into(),
        coinb2_hex: job["coinb2Hex"].as_str().unwrap().into(),
        merkle_steps_hex: job["merkleStepsHex"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_string())
            .collect(),
        version: job["version"].as_u64().unwrap() as u32,
        bits_hex: job["bitsHex"].as_str().unwrap().into(),
        ntime: submit["ntime"].as_u64().unwrap() as u32,
        ntime_hex: hex::encode((submit["ntime"].as_u64().unwrap() as u32).to_le_bytes()),
    };

    let submission = ShareSubmission {
        extranonce1: hex::decode(submit["extranonce1Hex"].as_str().unwrap()).unwrap(),
        extranonce2_hex: submit["extranonce2Hex"].as_str().unwrap().into(),
        ntime: submit["ntime"].as_u64().unwrap() as u32,
        nonce: submit["nonce"].as_u64().unwrap() as u32,
    };

    let recon = reconstruct_share(&stratum_job, &submission);
    let header_hex = hex::encode(recon.consensus_header);
    assert_eq!(
        header_hex,
        expected["consensusHeaderHex"].as_str().unwrap(),
        "consensus header mismatch"
    );

    let scratch = Scratchpad::new();
    let hash = scratch.hash(&recon.consensus_header);
    let scrypt_hex = hex::encode(hash);
    assert_eq!(
        scrypt_hex,
        expected["scrypt2Hex"].as_str().unwrap(),
        "scrypt2 hash mismatch"
    );
}
