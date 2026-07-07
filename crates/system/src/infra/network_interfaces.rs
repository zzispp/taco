use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::application::{SystemError, SystemResult};

const SHARED_IPV4_FIRST_OCTET: u8 = 100;
const SHARED_IPV4_SECOND_OCTET_START: u8 = 64;
const SHARED_IPV4_SECOND_OCTET_END: u8 = 127;
#[cfg(not(unix))]
const INTERFACE_IP_UNSUPPORTED_ERROR: &str = "infra.network.interface_ip_unsupported";

#[cfg(unix)]
pub(super) fn public_ips() -> SystemResult<Vec<String>> {
    use nix::ifaddrs::getifaddrs;

    let mut ips = Vec::new();
    for iface in getifaddrs().map_err(interface_error)? {
        let Some(address) = iface.address.and_then(|value| sockaddr_ip(&value)) else {
            continue;
        };
        push_public_ip(&mut ips, address);
    }
    Ok(ips)
}

#[cfg(unix)]
fn sockaddr_ip(address: &nix::sys::socket::SockaddrStorage) -> Option<IpAddr> {
    if let Some(ipv4) = address.as_sockaddr_in() {
        return Some(IpAddr::V4(ipv4.ip()));
    }
    address.as_sockaddr_in6().map(|ipv6| IpAddr::V6(ipv6.ip()))
}

#[cfg(not(unix))]
pub(super) fn public_ips() -> SystemResult<Vec<String>> {
    Err(SystemError::Infrastructure(INTERFACE_IP_UNSUPPORTED_ERROR.into()))
}

fn push_public_ip(ips: &mut Vec<String>, ip: IpAddr) {
    if ignored_ip(&ip) || internal_ip(&ip) {
        return;
    }
    let value = ip.to_string();
    if !ips.contains(&value) {
        ips.push(value);
    }
}

fn ignored_ip(ip: &IpAddr) -> bool {
    ip.is_loopback() || ip.is_unspecified() || ip.is_multicast()
}

fn internal_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(value) => internal_ipv4(value),
        IpAddr::V6(value) => internal_ipv6(value),
    }
}

fn internal_ipv4(ip: &Ipv4Addr) -> bool {
    ip.is_private() || ip.is_link_local() || shared_ipv4(ip)
}

fn internal_ipv6(ip: &Ipv6Addr) -> bool {
    ip.is_unique_local() || ip.is_unicast_link_local()
}

fn shared_ipv4(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] == SHARED_IPV4_FIRST_OCTET && (SHARED_IPV4_SECOND_OCTET_START..=SHARED_IPV4_SECOND_OCTET_END).contains(&octets[1])
}

#[cfg(unix)]
fn interface_error(error: nix::Error) -> SystemError {
    SystemError::Infrastructure(format!("interface IP collection failed: {error}"))
}
