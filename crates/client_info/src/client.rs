use std::net::{IpAddr, SocketAddr};

use http::{HeaderMap, HeaderName, header::USER_AGENT};

const UNKNOWN_BROWSER: &str = "Unknown";
const UNKNOWN_OS: &str = "Unknown";
const PROXY_HEADERS: [&str; 4] = ["x-forwarded-for", "proxy-client-ip", "wl-proxy-client-ip", "x-real-ip"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientInfo {
    pub ip: IpAddr,
    pub user_agent: String,
    pub browser: String,
    pub os: String,
}

impl ClientInfo {
    pub fn from_headers(headers: &HeaderMap, peer: SocketAddr) -> Self {
        let user_agent = header_text(headers, &USER_AGENT).unwrap_or_default().to_owned();
        Self {
            ip: forwarded_ip(headers).unwrap_or_else(|| peer.ip()),
            browser: browser_name(&user_agent).into(),
            os: os_name(&user_agent).into(),
            user_agent,
        }
    }

    pub fn ipaddr(&self) -> String {
        self.ip.to_string()
    }
}

fn forwarded_ip(headers: &HeaderMap) -> Option<IpAddr> {
    PROXY_HEADERS.iter().find_map(|name| valid_ip_from_header(headers, name))
}

fn valid_ip_from_header(headers: &HeaderMap, name: &'static str) -> Option<IpAddr> {
    let name = HeaderName::from_static(name);
    headers
        .get_all(name)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .find_map(parse_ip)
}

fn parse_ip(value: &str) -> Option<IpAddr> {
    let value = value.trim().trim_matches('"');
    if value.is_empty() || value.eq_ignore_ascii_case("unknown") {
        return None;
    }
    value.parse().ok()
}

fn header_text<'a>(headers: &'a HeaderMap, name: &HeaderName) -> Option<&'a str> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn browser_name(agent: &str) -> &'static str {
    if agent.contains("Edg/") {
        return "Edge";
    }
    if agent.contains("Firefox/") {
        return "Firefox";
    }
    if agent.contains("Chrome/") || agent.contains("CriOS/") {
        return "Chrome";
    }
    if agent.contains("Safari/") {
        return "Safari";
    }
    UNKNOWN_BROWSER
}

fn os_name(agent: &str) -> &'static str {
    if agent.contains("iPhone") || agent.contains("iPad") {
        return "iOS";
    }
    if agent.contains("Android") {
        return "Android";
    }
    if agent.contains("Windows") {
        return "Windows";
    }
    if agent.contains("Mac OS X") || agent.contains("Macintosh") {
        return "macOS";
    }
    if agent.contains("Linux") {
        return "Linux";
    }
    UNKNOWN_OS
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

    use http::{HeaderMap, HeaderValue, header::USER_AGENT};

    use super::ClientInfo;

    const PEER: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 7)), 8080);

    #[test]
    fn proxy_headers_follow_ruoyi_priority() {
        let headers = headers(&[
            ("x-forwarded-for", "203.0.113.7"),
            ("proxy-client-ip", "198.51.100.8"),
            ("wl-proxy-client-ip", "192.0.2.9"),
            ("x-real-ip", "192.0.2.10"),
        ]);

        assert_eq!(ClientInfo::from_headers(&headers, PEER).ip, "203.0.113.7".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn proxy_chain_uses_first_valid_ip() {
        let headers = headers(&[("x-forwarded-for", "unknown, invalid, 198.51.100.3, 203.0.113.4")]);

        assert_eq!(ClientInfo::from_headers(&headers, PEER).ip, "198.51.100.3".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn invalid_higher_priority_header_allows_next_header() {
        let headers = headers(&[("x-forwarded-for", "unknown, invalid"), ("proxy-client-ip", "2001:db8::5")]);

        assert_eq!(ClientInfo::from_headers(&headers, PEER).ip, "2001:db8::5".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn ipv6_is_preserved() {
        let headers = headers(&[("x-real-ip", "2001:db8:85a3::8a2e:370:7334")]);

        assert_eq!(
            ClientInfo::from_headers(&headers, PEER).ip,
            IpAddr::V6("2001:db8:85a3::8a2e:370:7334".parse::<Ipv6Addr>().unwrap())
        );
    }

    #[test]
    fn missing_or_invalid_proxy_headers_fall_back_to_peer() {
        for headers in [HeaderMap::new(), headers(&[("x-forwarded-for", "unknown, nope")])] {
            assert_eq!(ClientInfo::from_headers(&headers, PEER).ip, PEER.ip());
        }
    }

    #[test]
    fn user_agent_produces_browser_and_os() {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1 Safari/604.1"),
        );

        let info = ClientInfo::from_headers(&headers, PEER);

        assert_eq!(info.browser, "Safari");
        assert_eq!(info.os, "iOS");
        assert!(info.user_agent.starts_with("Mozilla/5.0"));
    }

    fn headers(values: &[(&'static str, &'static str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (name, value) in values {
            headers.insert(*name, HeaderValue::from_static(value));
        }
        headers
    }
}
