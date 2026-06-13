//! mDNS-based LAN peer discovery for Local Native P2P sync.
//!
//! Advertises the local RPC server on the network and discovers other
//! Local Native instances so users don't have to type IP addresses manually.

use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;
use std::net::IpAddr;

/// The mDNS service type used by Local Native instances.
pub const SERVICE_TYPE: &str = "_localnative._tcp.local.";

/// mDNS TXT record key for the Local Native version.
const TXT_VERSION: &str = "version";

/// Information about a discovered peer on the LAN.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerInfo {
    pub hostname: String,
    pub addresses: Vec<IpAddr>,
    pub port: u16,
    pub version: String,
}

/// Start advertising this Local Native instance on the LAN via mDNS.
///
/// Returns the [`ServiceDaemon`] handle which must be kept alive for the
/// duration of the advertisement. Call [`stop_advertising`] (or just drop
/// the daemon) to unregister.
pub fn start_advertising(port: u16) -> Result<ServiceDaemon, mdns_sd::Error> {
    let daemon = ServiceDaemon::new()?;

    let version = env!("CARGO_PKG_VERSION");
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let instance_name = format!("LocalNative-{}", hostname);

    let mut properties = HashMap::new();
    properties.insert(TXT_VERSION.to_string(), version.to_string());

    let service_info = ServiceInfo::new(
        SERVICE_TYPE,
        &instance_name,
        &format!("{}.", hostname),
        "",
        port,
        properties,
    )?;

    daemon.register(service_info)?;

    tracing::info!(port, %hostname, "mDNS: advertising Local Native on the LAN");
    Ok(daemon)
}

/// Stop advertising and shut down the mDNS daemon gracefully.
pub fn stop_advertising(daemon: ServiceDaemon) {
    match daemon.shutdown() {
        Err(e) => {
            tracing::warn!("mDNS: error during shutdown: {}", e);
        }
        _ => {
            tracing::info!("mDNS: stopped advertising");
        }
    }
}

/// Scan the LAN for other Local Native instances.
///
/// Browses for `duration` and returns all peers found. This is a blocking
/// async function that resolves after the timeout.
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

        match tokio::time::timeout(
            remaining,
            tokio::task::spawn_blocking({
                let receiver = receiver.clone();
                move || receiver.recv_timeout(std::time::Duration::from_millis(500))
            }),
        )
        .await
        {
            Ok(Ok(Ok(event))) => match event {
                ServiceEvent::ServiceResolved(info) => {
                    let version = info
                        .get_properties()
                        .get(TXT_VERSION)
                        .map(|v| v.val_str().to_string())
                        .unwrap_or_default();

                    let addresses: Vec<IpAddr> = info
                        .get_addresses()
                        .iter()
                        .map(|addr| addr.to_ip_addr())
                        .collect();

                    if !addresses.is_empty() {
                        let peer = PeerInfo {
                            hostname: info.get_hostname().trim_end_matches('.').to_string(),
                            addresses,
                            port: info.get_port(),
                            version,
                        };
                        peers.insert(info.get_fullname().to_string(), peer);
                    }
                }
                ServiceEvent::ServiceRemoved(_, fullname) => {
                    peers.remove(&fullname);
                }
                _ => {}
            },
            // Timeout on the recv — just loop and check deadline
            Ok(Ok(Err(_))) => continue,
            // spawn_blocking join error — unlikely, just continue
            Ok(Err(_)) => continue,
            // Overall timeout expired
            Err(_) => break,
        }
    }

    daemon.shutdown().ok();
    Ok(peers.into_values().collect())
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
    fn test_peer_info_clone() {
        let peer = PeerInfo {
            hostname: "test-host".to_string(),
            addresses: vec!["192.168.1.100".parse().unwrap()],
            port: 2345,
            version: "0.7.0".to_string(),
        };
        let cloned = peer.clone();
        assert_eq!(peer, cloned);
    }
}
