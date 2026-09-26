/*
    Local Native
    Copyright (C) 2018-2019  Yi Wang

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! mDNS-based LAN peer discovery for Local Native sync.
//!
//! Advertises the local sync server so other devices can find it without
//! typing IP addresses. Addresses are filled in by the mDNS daemon itself
//! ([`ServiceInfo::enable_addr_auto`]) — registering without an address and
//! without auto mode produces a service that never resolves.

use crate::wire::PROTOCOL_VERSION;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Mutex, OnceLock};

/// The mDNS service type used by Local Native instances.
pub const SERVICE_TYPE: &str = "_localnative._tcp.local.";

/// mDNS TXT record keys. `proto` is the sync protocol version — the crate
/// version says nothing about whether two devices can talk.
const TXT_PROTOCOL: &str = "proto";
const TXT_NODE: &str = "node";
const TXT_NAME: &str = "name";

/// The `node_id` of this database, used to filter ourselves out of results.
static OWN_NODE: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn own_node() -> &'static Mutex<Option<String>> {
    OWN_NODE.get_or_init(|| Mutex::new(None))
}

/// Remember this database's node id so [`discover_peers`] can skip its own
/// advertisement (set when the sync server starts).
pub fn set_own_node(node_id: &str) {
    *own_node()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(node_id.to_string());
}

/// A human-friendly device name from the hostname.
fn device_name() -> String {
    hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .trim_end_matches(".local")
        .trim_end_matches('.')
        .to_string()
}

/// A host name mdns-sd accepts: labels ending in `.local.`, with any suffix
/// the OS already appended stripped first (`Yis-MBP.local` → `Yis-MBP.local.`).
fn local_hostname_from(name: &str) -> String {
    let stripped = name.trim_end_matches(".local.").trim_end_matches('.');
    format!("{stripped}.local.")
}

fn local_hostname() -> String {
    local_hostname_from(&device_name())
}

/// Information about a discovered peer on the LAN.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerInfo {
    pub hostname: String,
    pub addresses: Vec<IpAddr>,
    pub port: u16,
    /// Sync protocol version, from the TXT record.
    pub protocol: u32,
    /// The peer's database node id (empty when it did not advertise one).
    pub node_id: String,
}

/// Start advertising this Local Native sync server on the LAN.
///
/// `node_id` is this database's node id: it makes the instance name unique
/// and lets [`discover_peers`] filter this device out of its own results.
///
/// Returns the [`ServiceDaemon`] handle which must be kept alive for the
/// duration of the advertisement; call [`stop_advertising`] (or drop it) to
/// unregister.
pub fn start_advertising(port: u16, node_id: &str) -> Result<ServiceDaemon, mdns_sd::Error> {
    let daemon = ServiceDaemon::new()?;
    set_own_node(node_id);

    let name = device_name();
    // The instance name must be unique on the network; the node id prefix
    // keeps two devices with the same hostname apart.
    let instance_name = format!("LocalNative-{}", &node_id[..12.min(node_id.len())]);

    let mut properties = HashMap::new();
    properties.insert(TXT_PROTOCOL.to_string(), PROTOCOL_VERSION.to_string());
    properties.insert(TXT_NODE.to_string(), node_id.to_string());
    properties.insert(TXT_NAME.to_string(), name);

    let service_info = ServiceInfo::new(
        SERVICE_TYPE,
        &instance_name,
        &local_hostname(),
        "",
        port,
        properties,
    )?
    .enable_addr_auto();

    daemon.register(service_info)?;
    tracing::info!(port, "mDNS: advertising Local Native on the LAN");
    Ok(daemon)
}

/// Stop advertising and shut down the mDNS daemon gracefully.
pub fn stop_advertising(daemon: ServiceDaemon) {
    match daemon.shutdown() {
        Err(e) => tracing::warn!("mDNS: error during shutdown: {}", e),
        _ => tracing::info!("mDNS: stopped advertising"),
    }
}

/// Scan the LAN for other Local Native sync servers for `duration`.
/// This device's own advertisement is excluded when its node id is known.
pub async fn discover_peers(
    duration: std::time::Duration,
) -> Result<Vec<PeerInfo>, mdns_sd::Error> {
    let daemon = ServiceDaemon::new()?;
    let receiver = daemon.browse(SERVICE_TYPE)?;
    let mut peers: HashMap<String, PeerInfo> = HashMap::new();

    let deadline = tokio::time::Instant::now() + duration;
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        // The channel is a blocking flume receiver; poll it off the executor.
        let receiver2 = receiver.clone();
        let event = tokio::time::timeout(
            remaining,
            tokio::task::spawn_blocking(move || {
                receiver2.recv_timeout(std::time::Duration::from_millis(500))
            }),
        )
        .await;
        match event {
            // Channel closed or the worker panicked: nothing more will arrive.
            Ok(Err(_)) | Ok(Ok(Err(mdns_sd::RecvTimeoutError::Disconnected))) => break,
            // No event within the slice: re-check the deadline.
            Ok(Ok(Err(mdns_sd::RecvTimeoutError::Timeout))) => continue,
            Err(_) => break, // overall deadline
            Ok(Ok(Ok(event))) => match event {
                ServiceEvent::ServiceResolved(info) => {
                    let protocol = info
                        .get_property_val_str(TXT_PROTOCOL)
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
                    let node_id = info
                        .get_property_val_str(TXT_NODE)
                        .unwrap_or_default()
                        .to_string();
                    let addresses: Vec<IpAddr> = info
                        .get_addresses()
                        .iter()
                        .map(|addr| addr.to_ip_addr())
                        .collect();
                    if !addresses.is_empty() {
                        peers.insert(
                            info.get_fullname().to_string(),
                            PeerInfo {
                                hostname: info.get_hostname().trim_end_matches('.').to_string(),
                                addresses,
                                port: info.get_port(),
                                protocol,
                                node_id,
                            },
                        );
                    }
                }
                ServiceEvent::ServiceRemoved(_, fullname) => {
                    peers.remove(&fullname);
                }
                _ => {}
            },
        }
    }

    daemon.shutdown().ok();
    let own = own_node()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    let mut found: Vec<PeerInfo> = peers.into_values().collect();
    if let Some(own) = own {
        found.retain(|peer| peer.node_id != own);
    }
    found.sort_by(|a, b| a.hostname.cmp(&b.hostname));
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_type_format() {
        assert!(SERVICE_TYPE.starts_with('_'));
        assert!(SERVICE_TYPE.ends_with(".local."));
        assert!(SERVICE_TYPE.contains("._tcp"));
    }

    #[test]
    fn test_local_hostname_always_valid() {
        for input in [
            "Yis-MBP.local",
            "Yis-MBP",
            "myhost.local.",
            "myhost",
            "a.b.c",
        ] {
            let host = local_hostname_from(input);
            assert!(host.ends_with(".local."), "{input} -> {host}");
            assert_ne!(host, ".local.", "{input} -> {host}");
        }
    }
}
