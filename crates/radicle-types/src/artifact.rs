use radicle_artifact_core::protocol::Status;
use serde::Serialize;
use ts_rs::TS;

/// Connection status and measured latency of one home relay.
#[derive(Clone, Serialize, TS, Debug)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "artifact/")]
pub struct RelayHealth {
    pub url: String,
    pub connected: bool,
    /// Round-trip latency in milliseconds, when a probe has landed.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "number", optional)]
    pub latency_ms: Option<u64>,
    /// Most recent connection error while disconnected.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub last_error: Option<String>,
}

/// Flattened snapshot of the artifact node's status for the UI.
///
/// The node's nested stat groups are flattened into one struct so the frontend
/// can render a flat list, except for the relays, which stay a list because
/// there is one entry per configured relay.
#[derive(Clone, Serialize, TS, Debug)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "artifact/")]
pub struct ArtifactNodeStatus {
    /// Endpoint the node serves on, as a `radiroh://<base32>` URL.
    pub endpoint_id: String,
    /// Unix timestamp, in seconds, when the node bound its socket.
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
    #[ts(type = "number")]
    pub connections_opened_total: u64,
    #[ts(type = "number")]
    pub connections_closed_total: u64,
    #[ts(type = "number")]
    pub connections_direct_total: u64,
    #[ts(type = "number")]
    pub holepunch_attempts: u64,
    #[ts(type = "number")]
    pub paths_direct: u64,
    #[ts(type = "number")]
    pub paths_relayed: u64,
    /// Bytes sent across the wire, including discovery frames.
    #[ts(type = "number")]
    pub out_bytes: u64,
    /// Bytes received across the wire, data only.
    #[ts(type = "number")]
    pub in_bytes: u64,
    /// Per-home-relay status. Empty before a relay is selected, or when
    /// relays are disabled.
    pub relays: Vec<RelayHealth>,
    /// Relay the node would prefer, when a net report has landed.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub preferred_relay: Option<String>,
    /// Whether a UDP round trip completed over IPv4, meaning direct UDP works.
    pub udp_v4: bool,
    /// Whether a UDP round trip completed over IPv6.
    pub udp_v6: bool,
    /// Set when no home relay is connected: peers that cannot holepunch may
    /// be unable to reach this node.
    pub relay_unreachable: bool,
    /// Where the node keeps the bytes it serves. A download writes the file
    /// the user asked for and keeps a copy here, so this is the second copy
    /// deleting the saved file does not remove.
    pub store_path: String,
}

impl ArtifactNodeStatus {
    pub fn new(status: Status, store_path: String) -> Self {
        Self {
            store_path,
            endpoint_id: status.endpoint_id.to_string(),
            started_at_unix: status.started_at_unix,
            seeded_count: status.seeded.count as u64,
            seeded_bytes_logical: status.seeded.bytes_logical,
            connections_active: status.connections.active,
            connections_opened_total: status.connections.opened_total,
            connections_closed_total: status.connections.closed_total,
            connections_direct_total: status.connections.direct_total,
            holepunch_attempts: status.connections.holepunch_attempts,
            paths_direct: status.connections.paths_direct,
            paths_relayed: status.connections.paths_relayed,
            out_bytes: status.traffic.out_bytes,
            in_bytes: status.traffic.in_bytes,
            relays: status
                .relay
                .relays
                .into_iter()
                .map(|relay| RelayHealth {
                    url: relay.url.to_string(),
                    connected: relay.connected,
                    latency_ms: relay.latency_ms,
                    last_error: relay.last_error,
                })
                .collect(),
            preferred_relay: status.relay.preferred,
            udp_v4: status.relay.udp_v4,
            udp_v6: status.relay.udp_v6,
            relay_unreachable: status.warnings.relay_unreachable,
        }
    }
}

/// One progress frame from an in-flight artifact download, flattened for the
/// UI and tagged with the content id so several downloads can share one event
/// channel.
#[derive(Clone, Serialize, TS, Debug)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "artifact/")]
pub struct ArtifactProgress {
    pub cid: String,
    /// One of `connecting`, `tryingLocation`, `locationFailed`, `downloading`
    /// or `exporting`.
    pub phase: String,
    /// Bytes moved so far, while downloading or exporting.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "number", optional)]
    pub offset: Option<u64>,
    /// Total bytes, when the node knows it.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "number", optional)]
    pub total: Option<u64>,
}

impl ArtifactProgress {
    pub fn new(cid: &str, progress: &radicle_artifact_core::protocol::FetchProgress) -> Self {
        use radicle_artifact_core::protocol::FetchProgress;

        let (phase, offset, total) = match progress {
            FetchProgress::Connecting => ("connecting", None, None),
            FetchProgress::TryingLocation { .. } => ("tryingLocation", None, None),
            FetchProgress::LocationFailed { .. } => ("locationFailed", None, None),
            FetchProgress::Downloading { offset, total } => ("downloading", Some(*offset), *total),
            FetchProgress::Exporting { offset, total, .. } => ("exporting", Some(*offset), *total),
        };

        Self {
            cid: cid.to_owned(),
            phase: phase.to_owned(),
            offset,
            total,
        }
    }
}
