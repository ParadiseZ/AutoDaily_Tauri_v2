use std::process::Command;
use std::io::{self, Read};
use crate::app::AppResult;
use super::adb_error::AdbError;

pub struct AdbClient {
    device_id: String,
}

impl AdbClient {
    pub fn new(device_id: &str) -> Self {
        AdbClient {
            device_id: device_id.to_string(),
        }
    }

    pub fn execute(&self, command: &str) -> AppResult<String> {
        let output = Command::new("adb")
            .arg("-s")
            .arg(&self.device_id)
            .args(command.split_whitespace())
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(AdbError::CommandExecutionFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into())
        }
    }

    pub fn capture_screenshot(&self) -> AppResult<Vec<u8>> {
        let mut child = Command::new("adb")
            .arg("-s")
            .arg(&self.device_id)
            .arg("exec-out")
            .arg("screencap")
            .arg("-p")
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        let mut screenshot = Vec::new();
        child.stdout
            .as_mut()
            .ok_or(AdbError::StdoutCaptureFailed)?
            .read_to_end(&mut screenshot)?;

        child.wait()?;
        Ok(screenshot)
    }
}

pub fn get_connected_devices() -> AppResult<Vec<String>> {
    let output = Command::new("adb")
        .arg("devices")
        .output()?;

    if output.status.success() {
        let devices = String::from_utf8_lossy(&output.stdout)
            .lines()
            .skip(1)
            .filter_map(|line| line.split_whitespace().next().map(|s| s.to_string()))
            .collect::<Vec<_>>();
            
        Ok(devices)
    } else {
        Err(AdbError::DeviceListingFailed(
            String::from_utf8_lossy(&output.stderr).to_string()
        ).into())
    }
}
