use crate::StatusResponse;

use std::{fmt, path::PathBuf, thread::JoinHandle};

pub(crate) mod cmd;
pub(crate) mod error;

pub struct Keybase {
    pub username: String,
    paperkey: String,
    pub keybase_path: PathBuf,
    pub status: StatusResponse,
    pub listen_threads: Vec<JoinHandle<()>>,
}

impl fmt::Debug for Keybase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Keybase {{ username: {}, status: {:?}, listen_threads: {} }}",
            self.username,
            self.status,
            self.listen_threads.len()
        )
    }
}

impl Keybase {
    pub async fn new(
        username: impl Into<String>,
        paperkey: impl Into<String>,
        opt_path: Option<PathBuf>,
    ) -> error::Result<Keybase> {
        let username: String = username.into();
        let paperkey: String = paperkey.into();
        // use use specified keybase path OR
        let keybase_path: PathBuf = match opt_path {
			Some(p) => p,
			None => {
				cmd::find_keybase().await?
			}
		};

        let keybase_status: StatusResponse = cmd::call_status(&keybase_path).await?;
        Ok(Keybase {
            username,
            paperkey,
            keybase_path,
            status: keybase_status,
            listen_threads: vec![],
        })
    }

    pub async fn logout(&mut self) -> error::Result<()> {
        let _output = cmd::exec(&self.keybase_path, &["logout"], None).await?;
        self.status = cmd::call_status(&self.keybase_path).await?;
        Ok(())
    }

    pub async fn login(&mut self) -> error::Result<()> {
        let _output = cmd::exec(
            &self.keybase_path,
            &["oneshot", "-u", &self.username.as_mut_str()],
            Some(self.paperkey.clone()),
        ).await?;
        self.status = cmd::call_status(&self.keybase_path).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Keybase;
    use std::{env, path::PathBuf, string::String};

    #[tokio::test]
    async fn can_create_keybase() {
        let k = Keybase::new("none", "none", None).await.unwrap();
        assert_eq!(k.username, String::from("none"));
        assert_eq!(k.paperkey, String::from("none"));
        assert_eq!(k.keybase_path, super::cmd::find_keybase().await.unwrap());
    }

    #[tokio::test]
    async fn cant_create_keybase() {
        let k = Keybase::new("none", "none", Some(PathBuf::from("/bin/false"))).await;
        assert!(k.is_err());
    }

    #[tokio::test]
    async fn can_print_keybase() {
        let k = Keybase::new("none", "none", None).await.unwrap();
        println!("{:?}", k);
    }

    #[tokio::test]
    async fn can_logout_then_login() {
        let ku = env::var("KEYBASE_USERNAME").unwrap();
        let kp = env::var("KEYBASE_PAPERKEY").unwrap();
        let mut k = Keybase::new(ku, kp, None).await.unwrap();

        let result = k.logout().await;
        assert!(!result.is_err());
        assert_eq!(k.status.logged_in, false);

        let result = k.login().await;
        assert!(!result.is_err());
        assert_eq!(k.status.logged_in, true);
    }
}
