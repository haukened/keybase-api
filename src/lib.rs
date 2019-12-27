use futures::channel::mpsc;
use serde::{Deserialize, Serialize, Deserializer};
use std::error::Error;
use std::{fmt, io};

pub(crate) mod keybase_cmd {
    use super::{ApiError, KBError, StatusResponse};
    use serde::{Deserialize, Serialize};
    use serde_json;
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Stdio};

    thread_local! {
        pub static KEYBASE: PathBuf = find_keybase();
    }

    #[derive(Deserialize, Serialize)]
    pub struct APIResult<T> {
        pub result: Option<T>,
        pub error: Option<KBError>,
    }

    pub fn find_keybase() -> PathBuf {
        let local_path = String::from_utf8(
            Command::new("which")
                .arg("keybase")
                .output()
                .expect("which is not installed")
                .stdout,
        )
        .expect("Output not in UTF-8");
        Path::new(local_path.trim()).to_path_buf()
    }

    pub fn call_status(keybase_path: &Path) -> Result<StatusResponse, ApiError> {
        let child_proc = exec(keybase_path, &["status", "-j"])?;
        let output = child_proc.wait_with_output()?;
        if !output.status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Keybase returned non-zero exit code",
            )
            .into());
        }
        let output = String::from_utf8(output.stdout)?;
        let res: StatusResponse = serde_json::from_str(&output)?;
        Ok(res)
    }

    pub fn call_version(keybase_path: &Path) -> Result<String, ApiError> {
        let child_proc = exec(keybase_path, &["version", "-S", "-f", "s"])?;
        let output = child_proc.wait_with_output()?;
        if !output.status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Keybase returned non-zero exit code",
            )
            .into());
        }
        let output = String::from_utf8(output.stdout)?;
        Ok(output)
    }

    pub fn exec<I, S>(keybase_path: &Path, args: I) -> Result<Child, std::io::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        Command::new(keybase_path)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusResponse {
    #[serde(rename = "Username")]
    pub username: String,
    #[serde(rename = "LoggedIn")]
    pub logged_in: bool,
    #[serde(rename = "Device")]
    pub device: DeviceResponse,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceResponse {
    #[serde(rename = "type")]
    type_: String,
    name: String,
    #[serde(rename = "deviceID")]
    device_id: String,
    #[serde(deserialize_with = "de_bool_from_int")]
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

#[derive(Debug)]
pub enum ApiError {
    Parsing(serde_json::error::Error),
    ParsingWithRaw(serde_json::error::Error, String),
    IOErr(io::Error),
    KBErr(KBError),
    UTF8Err(std::string::FromUtf8Error),
    ChannelErr(mpsc::SendError),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct KBError {
    pub code: i32,
    pub message: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ApiError {}

impl From<mpsc::SendError> for ApiError {
    fn from(error: mpsc::SendError) -> Self {
        ApiError::ChannelErr(error)
    }
}

impl From<std::string::FromUtf8Error> for ApiError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        ApiError::UTF8Err(error)
    }
}

impl From<serde_json::error::Error> for ApiError {
    fn from(error: serde_json::error::Error) -> Self {
        ApiError::Parsing(error)
    }
}

impl From<std::io::Error> for ApiError {
    fn from(error: std::io::Error) -> Self {
        ApiError::IOErr(error)
    }
}

#[cfg(test)]
mod tests {
    use super::keybase_cmd::*;

    #[test]
    fn can_find_keybase() {
        println!("Keybase is at: {:?}", find_keybase());
        assert!(!find_keybase().to_str().unwrap().is_empty());
    }

    #[test]
    fn can_find_version() {
        let kb_path = find_keybase();
        println!("Keybase is version {:?}", call_version(&kb_path).unwrap());
        assert!(!call_version(&kb_path).unwrap().is_empty());
    }
}
