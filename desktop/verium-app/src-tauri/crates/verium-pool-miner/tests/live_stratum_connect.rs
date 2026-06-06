//! Live pool connectivity smoke test (requires network).
//! Run: cargo test -p verium-pool-miner --test live_stratum_connect -- --ignored --nocapture

use verium_pool_miner::parse_stratum_url;
use verium_pool_miner::stratum::StratumClient;

#[test]
#[ignore = "requires network access to official pool"]
fn connect_subscribe_authorize_official_pool() {
    let (host, port) = parse_stratum_url("stratum+tcp://mine.vericonomy.com:3333").unwrap();
    let mut client = StratumClient::connect(&host, port).expect("connect");
    client.subscribe().expect("subscribe");
    client
        .authorize("VPM4YzmqhAzL22mTWtcQ5Kd529CLKZ7AcX.smoke", "x")
        .expect("authorize");

    let mut subscribed = false;
    let mut authorized = false;
    for _ in 0..20 {
        match client.read_event().expect("read") {
            None => std::thread::sleep(std::time::Duration::from_millis(100)),
            Some(verium_pool_miner::stratum::StratumEvent::Subscribed { .. }) => {
                subscribed = true;
            }
            Some(verium_pool_miner::stratum::StratumEvent::Authorized) => {
                authorized = true;
            }
            Some(verium_pool_miner::stratum::StratumEvent::SetDifficulty(_)) => {}
            Some(verium_pool_miner::stratum::StratumEvent::Notify(_)) => break,
            Some(verium_pool_miner::stratum::StratumEvent::SubmitResult { .. }) => {
                panic!("unexpected submit result during handshake")
            }
            Some(other) => panic!("unexpected event: {other:?}"),
        }
        if subscribed && authorized {
            break;
        }
    }
    assert!(subscribed, "never received subscribe response");
    assert!(authorized, "never received authorize response");
}
