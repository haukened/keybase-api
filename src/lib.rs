pub mod keybase;
pub use keybase::Keybase;

use serde::{Deserialize, Serialize, Deserializer};

pub(crate) mod keybase_cmd {
    use super::{keybase_error::*, StatusResponse};
    use serde_json;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::io::Write;

    pub fn find_keybase() -> Result<PathBuf> {
        let local_path = String::from_utf8(
            Command::new("which")
                .arg("keybase")
                .output()
                .expect("which is not installed")
                .stdout,
        )
        .expect("Output not in UTF-8");
        Ok(Path::new(local_path.trim()).to_path_buf())
    }

    pub fn call_status(keybase_path: &Path) -> Result<StatusResponse> {
        let output = exec(keybase_path, &["status", "-j"], None)?;
        let res: StatusResponse = serde_json::from_str(&output)?;
        Ok(res)
    }

    pub fn call_version(keybase_path: &Path) -> Result<String> {
        let output = exec(keybase_path, &["version", "-S", "-f", "s"], None)?;
        Ok(output)
    }

    pub fn exec<I, S>(keybase_path: &Path, args: I, stdin_to_write: Option<String>) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        let mut child_proc = Command::new(keybase_path)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        if let Some(stdin_found) = stdin_to_write {
            let child_stdin = child_proc.stdin.as_mut().expect("Failed to open stdin");
            child_stdin.write_all(stdin_found.as_bytes())?;
        }
        let output = child_proc.wait_with_output()?;
        if !output.status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Keybase returned non-zero exit code",
            ).into());
        }
        let output = String::from_utf8(output.stdout)?;
        Ok(output)
    }
}

fn default_device() -> DeviceResponse {
    DeviceResponse {
        type_: String::default(),
        name: String::default(),
        device_id: String::default(),
        status: false
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusResponse {
    #[serde(rename = "Username")]
    #[serde(default = "String::default")]
    pub username: String,
    #[serde(rename = "LoggedIn")]
    #[serde(default = "bool::default")]
    pub logged_in: bool,
    #[serde(rename = "Device")]
    #[serde(default = "default_device")]
    pub device: DeviceResponse,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceResponse {
    #[serde(rename = "type")]
    #[serde(default = "String::default")]
    type_: String,
    #[serde(default = "String::default")]
    name: String,
    #[serde(rename = "deviceID")]
    #[serde(default = "String::default")]
    device_id: String,
    #[serde(deserialize_with = "de_bool_from_int")]
    #[serde(default = "bool::default")]
    status: bool,
}

#[cfg_attr(tarpaulin, skip)]
fn de_bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where D: Deserializer<'de>
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}

#[macro_use]
extern crate error_chain;

pub mod keybase_error {
    error_chain! {
        foreign_links {
            Parsing(::serde_json::error::Error);
            IOErr(::std::io::Error);
            UTF8Err(std::string::FromUtf8Error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::keybase_cmd::*;

    #[test]
    fn can_find_keybase() {
        println!("Keybase is at: {:?}", find_keybase());
        assert!(!find_keybase().unwrap().to_str().unwrap().is_empty());
    }

    #[test]
    fn can_find_version() {
        let kb_path = find_keybase().unwrap();
        println!("Keybase is version {:?}", call_version(&kb_path).unwrap());
        assert!(!call_version(&kb_path).unwrap().is_empty());
    }

    #[test]
    fn cant_find_version() {
        let kb_fakepath = std::path::Path::new("/bin/false").to_path_buf();
        assert!(call_version(&kb_fakepath).is_err());
    }

    #[test]
    fn can_get_status() {
        let kb_path = find_keybase().unwrap();
        let kb_status = call_status(&kb_path).unwrap();
        // ensure the value must be true or false - this will fail if the device is not provisioned
        // As the DeviceResponse struct will come from a JSON null entry and panic.
        assert!(kb_status.logged_in == false || kb_status.logged_in == true);
    }

    #[test]
    fn cant_get_status() {
        let kb_fakepath = std::path::Path::new("/bin/false").to_path_buf();
        assert!(call_status(&kb_fakepath).is_err());
    }

    #[test]
    fn cant_exec_command() {
        let kb_fakepath = std::path::Path::new("/none/abcde").to_path_buf();
        assert!(exec(&kb_fakepath, &["none", "nil", "nada"], None).is_err())
    }
}
