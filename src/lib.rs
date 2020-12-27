pub mod keybase;
pub use keybase::Keybase;

use serde::{Deserialize, Deserializer, Serialize};

#[cfg(not(unix))]
compile_error!("sorry. this library currently depends heavily on native unix utilities, and thus will not compile on other platforms");

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusResponse {
    #[serde(rename = "Username")]
    #[serde(default)]
    pub username: String,
    #[serde(rename = "LoggedIn")]
    #[serde(default)]
    pub logged_in: bool,
    #[serde(rename = "Device")]
    #[serde(deserialize_with = "parse_device_response")]
    pub device: DeviceResponse,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceResponse {
    #[serde(rename = "type")]
    #[serde(default)]
    type_: String,
    #[serde(default)]
    name: String,
    #[serde(rename = "deviceID")]
    #[serde(default)]
    device_id: String,
    #[serde(deserialize_with = "de_bool_from_int")]
    #[serde(default = "bool::default")]
    status: bool,
}

impl Default for DeviceResponse {
    fn default() -> Self {
        DeviceResponse {
            type_: String::default(),
            name: String::default(),
            device_id: String::default(),
            status: false,
        }
    }
}

fn parse_device_response<'de, D>(deserializer: D) -> Result<DeviceResponse, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(|x: Option<_>| x.unwrap_or_else(Default::default))
}

#[cfg_attr(tarpaulin, skip)]
fn de_bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
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

#[cfg(test)]
mod tests {
    use crate::keybase::cmd::*;

    #[tokio::test]
    async fn can_find_keybase() {
		let search_result = find_keybase().await;
        println!("Keybase is at: {:?}", search_result);
        assert!(!search_result.unwrap().to_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn can_find_version() {
        let kb_search_result = find_keybase().await.unwrap();
		let version_result = call_version(&kb_search_result).await.unwrap();
        println!("Keybase is version {:?}", version_result);
        assert!(!version_result.is_empty());
    }

    #[tokio::test]
    async fn cant_find_version() {
        let kb_fakepath = std::path::Path::new("/bin/false").to_path_buf();
        assert!(call_version(&kb_fakepath).await.is_err());
    }

    #[tokio::test]
    async fn can_get_status() {
        let kb_path = find_keybase().await.unwrap();
        let kb_status = call_status(&kb_path).await.unwrap();
        // ensure the value must be true or false - this will fail if the device is not provisioned
        // As the DeviceResponse struct will come from a JSON null entry and panic.
        assert!(kb_status.logged_in == false || kb_status.logged_in == true);
    }

    #[tokio::test]
    async fn cant_get_status() {
        let kb_fakepath = std::path::Path::new("/bin/false").to_path_buf();
        assert!(call_status(&kb_fakepath).await.is_err());
    }

    #[tokio::test]
    async fn cant_exec_command() {
        let kb_fakepath = std::path::Path::new("/none/abcde").to_path_buf();
        assert!(exec(&kb_fakepath, &["none", "nil", "nada"], None).await.is_err())
    }
}
