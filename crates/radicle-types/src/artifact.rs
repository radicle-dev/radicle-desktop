use radicle_artifact_core::protocol::Status;
use serde::Serialize;
use ts_rs::TS;

/// Flattened snapshot of the rad-artifact node's status for the UI. Mirrors
/// every field of the node's `Status` reply; the nested stat groups are
/// flattened so the frontend can render one flat list.
#[derive(Clone, Serialize, TS, Debug)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "artifact/")]
pub struct ArtifactNodeStatus {
    /// Endpoint id the node serves on, as a `radiroh://<base32>` URL.
    pub endpoint_id: String,
    /// Unix timestamp (seconds) when the node bound its socket.
    #[ts(type = "number")]
    pub started_at_unix: i64,
    /// Number of seeded `(rid, cid)` pairs.
    #[ts(type = "number")]
    pub seeded_count: u64,
    /// Sum of logical artifact sizes across all seeded tags, in bytes.
    #[ts(type = "number")]
    pub seeded_bytes_logical: u64,
    /// Currently open connections.
    #[ts(type = "number")]
    pub connections_active: u32,
    /// Lifetime opened-handshaked connection count.
    #[ts(type = "number")]
    pub connections_opened_total: u64,
    /// Lifetime closed connection count.
    #[ts(type = "number")]
    pub connections_closed_total: u64,
    /// Lifetime direct (non-relayed) connection count.
    #[ts(type = "number")]
    pub connections_direct_total: u64,
    /// Lifetime holepunch attempts.
    #[ts(type = "number")]
    pub holepunch_attempts: u64,
    /// Path counter: direct.
    #[ts(type = "number")]
    pub paths_direct: u64,
    /// Path counter: relayed.
    #[ts(type = "number")]
    pub paths_relayed: u64,
    /// Bytes sent across the wire (includes disco frames).
    #[ts(type = "number")]
    pub out_bytes: u64,
    /// Bytes received across the wire (data only).
    #[ts(type = "number")]
    pub in_bytes: u64,
}

impl From<Status> for ArtifactNodeStatus {
    fn from(s: Status) -> Self {
        Self {
            endpoint_id: s.endpoint_id.to_string(),
            started_at_unix: s.started_at_unix,
            seeded_count: s.seeded.count as u64,
            seeded_bytes_logical: s.seeded.bytes_logical,
            connections_active: s.connections.active,
            connections_opened_total: s.connections.opened_total,
            connections_closed_total: s.connections.closed_total,
            connections_direct_total: s.connections.direct_total,
            holepunch_attempts: s.connections.holepunch_attempts,
            paths_direct: s.connections.paths_direct,
            paths_relayed: s.connections.paths_relayed,
            out_bytes: s.traffic.out_bytes,
            in_bytes: s.traffic.in_bytes,
        }
    }
}
