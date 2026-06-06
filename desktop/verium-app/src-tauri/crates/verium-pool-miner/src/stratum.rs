//! Blocking Stratum TCP client (newline-delimited JSON).

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

use serde_json::{json, Value};

use crate::protocol::{reverse_buffer, StratumJob};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingKind {
    Subscribe,
    Authorize,
    Submit,
}

#[derive(Debug, Clone)]
pub struct StratumNotify {
    pub job: StratumJob,
    pub clean_jobs: bool,
}

#[derive(Debug)]
pub enum StratumEvent {
    Subscribed {
        extranonce1_hex: String,
        extranonce2_size: usize,
    },
    Authorized,
    SetDifficulty(f64),
    Notify(StratumNotify),
    SubmitResult {
        id: u64,
        accepted: bool,
        error: Option<String>,
    },
    Disconnected(String),
}

pub struct StratumClient {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    next_id: u64,
    user_agent: String,
    pending: HashMap<u64, PendingKind>,
}

impl StratumClient {
    pub fn connect(host: &str, port: u16) -> Result<Self, String> {
        let addr = format!("{host}:{port}");
        let stream = TcpStream::connect(&addr).map_err(|e| format!("connect {addr}: {e}"))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(120)))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_secs(30)))
            .map_err(|e| e.to_string())?;
        let reader = BufReader::new(
            stream
                .try_clone()
                .map_err(|e| format!("clone socket: {e}"))?,
        );
        Ok(Self {
            stream,
            reader,
            next_id: 1,
            user_agent: "vericonomy-wallet/1.0".to_string(),
            pending: HashMap::new(),
        })
    }

    pub fn subscribe(&mut self) -> Result<u64, String> {
        let id = self.next_id;
        self.next_id += 1;
        self.pending.insert(id, PendingKind::Subscribe);
        self.send(&json!({
            "id": id,
            "method": "mining.subscribe",
            "params": [&self.user_agent]
        }))
    }

    pub fn authorize(&mut self, username: &str, password: &str) -> Result<u64, String> {
        let id = self.next_id;
        self.next_id += 1;
        self.pending.insert(id, PendingKind::Authorize);
        self.send(&json!({
            "id": id,
            "method": "mining.authorize",
            "params": [username, password]
        }))
    }

    pub fn submit_share(
        &mut self,
        username: &str,
        job_id: &str,
        extranonce2_hex: &str,
        ntime_hex: &str,
        nonce_hex: &str,
    ) -> Result<u64, String> {
        let id = self.next_id;
        self.next_id += 1;
        self.pending.insert(id, PendingKind::Submit);
        self.send(&json!({
            "id": id,
            "method": "mining.submit",
            "params": [username, job_id, extranonce2_hex, ntime_hex, nonce_hex]
        }))
    }

    fn send(&mut self, msg: &Value) -> Result<u64, String> {
        let id = msg.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
        let line = serde_json::to_string(msg).map_err(|e| e.to_string())? + "\n";
        self.stream
            .write_all(line.as_bytes())
            .map_err(|e| format!("stratum write: {e}"))?;
        self.stream.flush().map_err(|e| e.to_string())?;
        Ok(id)
    }

    pub fn read_event(&mut self) -> Result<Option<StratumEvent>, String> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => return Ok(Some(StratumEvent::Disconnected("eof".into()))),
            Ok(_) => {}
            Err(e) => {
                return Ok(Some(StratumEvent::Disconnected(e.to_string())));
            }
        }
        let line = line.trim();
        if line.is_empty() {
            return Ok(None);
        }
        let msg: Value = serde_json::from_str(line).map_err(|e| format!("json: {e}"))?;
        Ok(Some(self.parse_message(&msg)))
    }

    fn parse_message(&mut self, msg: &Value) -> StratumEvent {
        if let Some(method) = msg.get("method").and_then(|m| m.as_str()) {
            return match method {
                "mining.set_difficulty" => {
                    let diff = msg
                        .get("params")
                        .and_then(|p| p.get(0))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0);
                    StratumEvent::SetDifficulty(diff)
                }
                "mining.notify" => parse_notify(msg),
                _ => StratumEvent::Disconnected(format!("unknown method {method}")),
            };
        }

        if let Some(id) = msg.get("id").and_then(|v| v.as_u64()) {
            if let Some(kind) = self.pending.remove(&id) {
                return match kind {
                    PendingKind::Subscribe => parse_subscribe_result(msg),
                    PendingKind::Authorize => {
                        let ok = msg.get("result").and_then(|r| r.as_bool()).unwrap_or(false);
                        if ok {
                            StratumEvent::Authorized
                        } else {
                            StratumEvent::Disconnected(format!(
                                "authorize rejected: {:?}",
                                msg.get("error")
                            ))
                        }
                    }
                    PendingKind::Submit => {
                        let accepted = msg.get("result").and_then(|r| r.as_bool()).unwrap_or(false);
                        let error = msg
                            .get("error")
                            .and_then(|e| e.as_array())
                            .map(|a| format!("{a:?}"));
                        StratumEvent::SubmitResult {
                            id,
                            accepted,
                            error,
                        }
                    }
                };
            }
        }

        StratumEvent::Disconnected("unhandled message".into())
    }
}

fn parse_subscribe_result(msg: &Value) -> StratumEvent {
    let result = msg.get("result");
    if let Some(arr) = result.and_then(|r| r.as_array()) {
        if arr.len() >= 3 && arr[0].is_array() {
            let en1 = arr[1].as_str().unwrap_or("").to_string();
            let en2_size = arr[2].as_u64().unwrap_or(4) as usize;
            return StratumEvent::Subscribed {
                extranonce1_hex: en1,
                extranonce2_size: en2_size,
            };
        }
    }
    StratumEvent::Disconnected(format!("bad subscribe response: {msg}"))
}

fn parse_notify(msg: &Value) -> StratumEvent {
    let p = msg.get("params").and_then(|v| v.as_array());
    if let Some(p) = p {
        if p.len() >= 9 {
            let prev_internal = p[1].as_str().unwrap_or("");
            let prev_rev = reverse_buffer(&hex::decode(prev_internal).unwrap_or_default());
            let prev_hash_hex = hex::encode(&prev_rev);
            let ntime_hex = p[7].as_str().unwrap_or("00000000").to_string();
            let ntime_bytes = hex::decode(&ntime_hex).unwrap_or_else(|_| vec![0; 4]);
            let ntime = if ntime_bytes.len() >= 4 {
                u32::from_le_bytes(ntime_bytes[..4].try_into().unwrap())
            } else {
                0
            };
            let merkle: Vec<String> = p[4]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let version =
                u32::from_str_radix(p[5].as_str().unwrap_or("00000004"), 16).unwrap_or(4);
            let job = StratumJob {
                job_id: p[0].as_str().unwrap_or("").to_string(),
                prev_hash_hex,
                coinb1_hex: p[2].as_str().unwrap_or("").to_string(),
                coinb2_hex: p[3].as_str().unwrap_or("").to_string(),
                merkle_steps_hex: merkle,
                version,
                bits_hex: p[6].as_str().unwrap_or("").to_string(),
                ntime,
                ntime_hex,
            };
            let clean = p[8].as_bool().unwrap_or(false);
            return StratumEvent::Notify(StratumNotify {
                job,
                clean_jobs: clean,
            });
        }
    }
    StratumEvent::Disconnected("bad notify".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscribe_result_parses_extranonce() {
        let msg: Value = serde_json::from_str(
            r#"{"id":1,"result":[[["mining.set_difficulty","x"],["mining.notify","x"]],"aabbccdd",4],"error":null}"#,
        )
        .unwrap();
        match parse_subscribe_result(&msg) {
            StratumEvent::Subscribed {
                extranonce1_hex,
                extranonce2_size,
            } => {
                assert_eq!(extranonce1_hex, "aabbccdd");
                assert_eq!(extranonce2_size, 4);
            }
            other => panic!("expected Subscribed, got {other:?}"),
        }
    }
}
