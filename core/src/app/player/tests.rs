use super::connection::format_ggrs_addr;
use super::sanitize_game_id;

#[test]
fn format_ggrs_addr_replaces_port_for_ipv4_socket_addr() {
    assert_eq!(format_ggrs_addr("127.0.0.1:1234", 7000), "127.0.0.1:7000");
}

#[test]
fn format_ggrs_addr_appends_port_for_ipv4_host() {
    assert_eq!(format_ggrs_addr("127.0.0.1", 7000), "127.0.0.1:7000");
}

#[test]
fn format_ggrs_addr_appends_port_for_hostname() {
    assert_eq!(format_ggrs_addr("localhost", 7000), "localhost:7000");
}

#[test]
fn format_ggrs_addr_replaces_port_for_hostname_with_port() {
    assert_eq!(format_ggrs_addr("localhost:1234", 7000), "localhost:7000");
}

#[test]
fn format_ggrs_addr_appends_port_for_ipv6_host_without_brackets() {
    assert_eq!(format_ggrs_addr("::1", 7000), "[::1]:7000");
    assert_eq!(format_ggrs_addr("2001:db8::1", 7000), "[2001:db8::1]:7000");
}

#[test]
fn format_ggrs_addr_replaces_port_for_ipv6_socket_addr() {
    assert_eq!(format_ggrs_addr("[::1]:1234", 7000), "[::1]:7000");
}

#[test]
fn format_ggrs_addr_replaces_port_for_ipv6_without_brackets_port_form() {
    assert_eq!(
        format_ggrs_addr("2001:db8::1:1234", 7000),
        "[2001:db8::1:1234]:7000"
    );
}

#[test]
fn sanitize_game_id_never_returns_empty() {
    assert!(!sanitize_game_id("").is_empty());
    assert!(!sanitize_game_id("日本語ゲーム").is_empty());
}
