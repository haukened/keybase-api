use crate::{keybase::error, StatusResponse};
use serde_json;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncWriteExt;

pub async fn find_keybase() -> error::Result<PathBuf> {
    let local_path = String::from_utf8(
        Command::new("which")
            .arg("keybase")
            .output()
            .await
            .expect("which is not installed")
            .stdout,
    )
    .expect("Output not in UTF-8");
    Ok(Path::new(local_path.trim()).to_path_buf())
}

pub async fn call_status(keybase_path: &Path) -> error::Result<StatusResponse> {
    let output = exec(keybase_path, &["status", "-j"], None).await?;
    let res: StatusResponse = serde_json::from_str(&output)?;
    Ok(res)
}

pub async fn call_version(keybase_path: &Path) -> error::Result<String> {
    let output = exec(keybase_path, &["version", "-S", "-f", "s"], None).await?;
    Ok(output)
}

pub async fn exec<I, S>(keybase_path: &Path, args: I, stdin_to_write: Option<String>) -> error::Result<String>
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
        child_stdin.write_all(stdin_found.as_bytes()).await?;
    }
    let output = child_proc.wait_with_output().await?;
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
