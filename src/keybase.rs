use crate::keybase_cmd;
use super::StatusResponse;
use std::{
    fmt,
    path::PathBuf,
    thread::JoinHandle,
};

use super::keybase_error::*;

pub struct Keybase {
    pub username: String,
    paperkey: String,
    pub keybase_path: PathBuf,
    pub status: StatusResponse,
    pub listen_threads: Vec<JoinHandle<()>>,
}

impl fmt::Debug for Keybase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!( f, "Keybase {{ username: {}, status: {:?}, listen_threads: {} }}",
            self.username,
            self.status,
            self.listen_threads.len()
        )
    }
}

impl Keybase {
    pub fn new<S>(username: S, paperkey: S, opt_path: Option<PathBuf>) -> Result<Keybase> 
    where 
        S: Into<String>,
    {
        let username: String = username.into();
        let paperkey: String = paperkey.into();
        // use use specified keybase path OR
        let keybase_path: PathBuf = opt_path.unwrap_or(
            // use `which` to find the keybase binary OR
            keybase_cmd::find_keybase()?
            );
        let keybase_status: StatusResponse = keybase_cmd::call_status(&keybase_path)?;
        Ok(Keybase {
            username,
            paperkey,
            keybase_path,
            status: keybase_status,
            listen_threads: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Keybase;
    use std::{
        string::String,
        path::PathBuf,
    };

    #[test]
    fn can_create_keybase() {
        let k = Keybase::new("none", "none", None).unwrap();
        assert_eq!(k.username, String::from("none"));
        assert_eq!(k.paperkey, String::from("none"));
        assert_eq!(k.keybase_path, super::keybase_cmd::find_keybase().unwrap());
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
}
