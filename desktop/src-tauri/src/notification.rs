use chrono::Local;
use std::process::Command;

const UNKNOWN_SSID: &str = "Unknown Wi-Fi";

pub fn connect_success_body() -> String {
    let ssid = current_ssid().unwrap_or_else(|| UNKNOWN_SSID.to_owned());
    let time = Local::now().format("%H:%M").to_string();
    format_connect_success_body(&ssid, &time)
}

fn format_connect_success_body(ssid: &str, time: &str) -> String {
    format!("Wi-Fi: {ssid} | Time: {time}")
}

#[cfg(target_os = "windows")]
fn current_ssid() -> Option<String> {
    run_command("netsh", &["wlan", "show", "interfaces"])
        .and_then(|output| parse_windows_ssid(&output))
}

#[cfg(target_os = "macos")]
fn current_ssid() -> Option<String> {
    let hardware_ports = run_command("networksetup", &["-listallhardwareports"])?;
    let mut is_wifi_port = false;
    let mut device = None;

    for line in hardware_ports.lines() {
        let line = line.trim();
        if let Some(port) = line.strip_prefix("Hardware Port:") {
            let port = port.trim();
            is_wifi_port =
                port.eq_ignore_ascii_case("Wi-Fi") || port.eq_ignore_ascii_case("AirPort");
        } else if is_wifi_port {
            if let Some(name) = line.strip_prefix("Device:") {
                device = nonempty(name.trim()).map(str::to_owned);
                break;
            }
        }
    }

    let device = device?;
    let network = run_command("networksetup", &["-getairportnetwork", &device])?;
    parse_current_wifi_network(&network)
}

#[cfg(target_os = "linux")]
fn current_ssid() -> Option<String> {
    let from_network_manager = run_command("nmcli", &["-t", "-f", "IN-USE,SSID", "device", "wifi"])
        .and_then(|output| parse_nmcli_ssid(&output));

    from_network_manager.or_else(|| {
        run_command("iwgetid", &["-r"])
            .and_then(|output| nonempty(output.trim()).map(str::to_owned))
    })
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn current_ssid() -> Option<String> {
    None
}

fn run_command(program: &str, args: &[&str]) -> Option<String> {
    let mut command = Command::new(program);
    command.args(args);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    let output = command.output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(any(target_os = "windows", test))]
fn parse_windows_ssid(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let (key, value) = line.trim().split_once(':')?;
        key.trim()
            .eq_ignore_ascii_case("SSID")
            .then(|| nonempty(value.trim()).map(str::to_owned))
            .flatten()
    })
}

fn parse_nmcli_ssid(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let (in_use, value) = line.split_once(':')?;
        if in_use != "*" {
            return None;
        }

        let decoded = decode_nmcli_value(value);
        nonempty(&decoded).map(str::to_owned)
    })
}

fn decode_nmcli_value(value: &str) -> String {
    let mut decoded = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character == '\\' {
            match chars.next() {
                Some(next @ (':' | '\\')) => decoded.push(next),
                Some(next) => {
                    decoded.push(character);
                    decoded.push(next);
                }
                None => decoded.push(character),
            }
        } else {
            decoded.push(character);
        }
    }
    decoded
}

#[cfg(any(target_os = "macos", test))]
fn parse_current_wifi_network(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let (label, value) = line.split_once(':')?;
        label
            .trim()
            .eq_ignore_ascii_case("Current Wi-Fi Network")
            .then(|| nonempty(value.trim()).map(str::to_owned))
            .flatten()
    })
}

fn nonempty(value: &str) -> Option<&str> {
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::{
        decode_nmcli_value, format_connect_success_body, parse_current_wifi_network,
        parse_nmcli_ssid, parse_windows_ssid,
    };

    #[test]
    fn success_body_matches_the_legacy_toast_detail_line() {
        assert_eq!(
            format_connect_success_body("Campus Wi-Fi", "09:08"),
            "Wi-Fi: Campus Wi-Fi | Time: 09:08"
        );
    }

    #[test]
    fn windows_parser_reads_ssid_and_ignores_bssid() {
        let output = "Interface name : Wi-Fi\nSSID : Campus Network\nBSSID : 01:23:45:67:89:ab\n";
        assert_eq!(
            parse_windows_ssid(output).as_deref(),
            Some("Campus Network")
        );
    }

    #[test]
    fn nmcli_parser_uses_active_network_and_unescapes_colons() {
        let output = "*:Campus\\: Guest\n:Other Network\n";
        assert_eq!(parse_nmcli_ssid(output).as_deref(), Some("Campus: Guest"));
    }

    #[test]
    fn nmcli_unescape_preserves_unrecognized_backslash_sequences() {
        assert_eq!(decode_nmcli_value(r"Campus\Guest"), r"Campus\Guest");
    }

    #[test]
    fn macos_parser_reads_the_current_wifi_network() {
        let output = "Current Wi-Fi Network: Campus Wi-Fi\n";
        assert_eq!(
            parse_current_wifi_network(output).as_deref(),
            Some("Campus Wi-Fi")
        );
    }
}
