use crate::{keybase::error::*, StatusResponse};

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
    pub fn new(
        username: impl Into<String>,
        paperkey: impl Into<String>,
        opt_path: Option<PathBuf>,
    ) -> Result<Keybase> {
        let username: String = username.into();
        let paperkey: String = paperkey.into();
        // use use specified keybase path OR
        let keybase_path: PathBuf = opt_path
            .ok_or_else(|| {
                // use `which` to find the keybase binary OR
                cmd::find_keybase()
            })
            .or_else(|e| e)?;

        let keybase_status: StatusResponse = cmd::call_status(&keybase_path)?;
        Ok(Keybase {
            username,
            paperkey,
            keybase_path,
            status: keybase_status,
            listen_threads: vec![],
        })
    }

    pub fn logout(&mut self) -> Result<()> {
        let _output = cmd::exec(&self.keybase_path, &["logout"], None)?;
        self.status = cmd::call_status(&self.keybase_path)?;
        Ok(())
    }

    pub fn login(&mut self) -> Result<()> {
        let _output = cmd::exec(
            &self.keybase_path,
            &["oneshot", "-u", &self.username.as_mut_str()],
            Some(self.paperkey.clone()),
        )?;
        self.status = cmd::call_status(&self.keybase_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Keybase;
    use std::{env::var, path::PathBuf, string::String};

    #[test]
    fn can_create_keybase() {
        let k = Keybase::new("none", "none", None).unwrap();
        assert_eq!(k.username, String::from("none"));
        assert_eq!(k.paperkey, String::from("none"));
        assert_eq!(k.keybase_path, super::cmd::find_keybase().unwrap());
    }

    #[test]
    fn cant_create_keybase() {
        let k = Keybase::new("none", "none", Some(PathBuf::from("/bin/false")));
        assert!(k.is_err());
    }

    #[test]
    fn can_print_keybase() {
        let k = Keybase::new("none", "none", None).unwrap();
        println!("{:?}", k);
    }

    #[test]
    fn can_logout_then_login() {
        let ku = var("KEYBASE_USERNAME").unwrap();
        let kp = var("KEYBASE_PAPERKEY").unwrap();
        let mut k = Keybase::new(ku, kp, None).unwrap();

        let result = k.logout();
        assert!(!result.is_err());
        assert_eq!(k.status.logged_in, false);

        let result = k.login();
        assert!(!result.is_err());
        assert_eq!(k.status.logged_in, true);
    }
}
