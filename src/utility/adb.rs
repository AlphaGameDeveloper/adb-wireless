use super::{error::CliError, pair::DeviceInfo, port::PortMapping};

pub fn adb_ensure_running() -> Result<(), CliError> {
    // Check if ADB exists
    match std::process::Command::new("adb").arg("version").output() {
        Ok(output) => {
            if !output.status.success() {
                return Err(CliError::AdbNotFound);
            }
        }
        Err(_) => return Err(CliError::AdbNotFound),
    }

    // Start ADB server
    let mut cmd = std::process::Command::new("adb");
    cmd.arg("start-server");

    let mut child = cmd.spawn().map_err(|err| CliError::AdbServerError(err))?;

    match child.wait() {
        Ok(status) => {
            if !status.success() {
                return Err(CliError::AdbServerError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to start ADB server.".to_string(),
                )));
            }
        }
        Err(err) => return Err(CliError::AdbServerError(err)),
    }

    Ok(())
}

pub fn adb_list_devices() -> Result<Vec<String>, CliError> {
    let output = std::process::Command::new("adb")
        .arg("devices")
        .output()
        .map_err(|err| CliError::AdbServerError(err))?;

    if !output.status.success() {
        return Err(CliError::AdbServerError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "Failed to list devices. {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let devices = stdout
        .lines()
        .skip(1) // Skip the first line (header)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == "device" {
                Some(parts[0].to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(devices)
}

pub fn adb_reverse_port(device_name: &str, mapping: &PortMapping) -> Result<(), CliError> {
    match std::process::Command::new("adb")
        .arg("-s")
        .arg(device_name)
        .arg("reverse")
        .arg(format!("tcp:{}", mapping.device_port))
        .arg(format!("tcp:{}", mapping.host_port))
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                return Err(CliError::AdbServerError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Failed to reverse port {}:{}. {}",
                        mapping.device_port,
                        mapping.host_port,
                        String::from_utf8_lossy(&output.stderr)
                    ),
                )));
            }
        }
        Err(err) => return Err(CliError::AdbServerError(err)),
    }

    Ok(())
}

pub fn adb_connect_device(device: &DeviceInfo, password: &str) -> Result<(), CliError> {
    match std::process::Command::new("adb")
        .arg("pair")
        .arg(format!("{}:{}", device.address, device.pairing_port))
        .arg(password)
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                return Err(CliError::AdbServerError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Failed to pair with device at {}:{}. {}",
                        device.address,
                        device.pairing_port,
                        String::from_utf8_lossy(&output.stderr)
                    ),
                )));
            }
        }
        Err(err) => return Err(CliError::AdbServerError(err)),
    }

    let mut cmd = std::process::Command::new("adb");
    cmd.arg("connect")
        .arg(format!("{}:{}", device.address, device.debugging_port));

    let mut child = cmd.spawn().map_err(|err| CliError::AdbServerError(err))?;

    match child.wait() {
        Ok(status) => {
            if !status.success() {
                return Err(CliError::AdbServerError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Failed to connect to device at {}:{}",
                        device.address, device.debugging_port,
                    ),
                )));
            }
        }
        Err(err) => return Err(CliError::AdbServerError(err)),
    }

    Ok(())
}
