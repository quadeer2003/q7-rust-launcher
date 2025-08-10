use std::io::Result;
use std::env;
use std::process::{Command, Stdio};

/// Spawn a non-blocking shell command using `sh -lc`.
pub fn run_shell(cmd: &str) -> Result<()> {
    #[cfg(windows)]
    {
        Command::new("cmd").arg("/C").arg(cmd).spawn()?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let mut c = Command::new("sh");
        c.arg("-lc").arg(cmd);
        if env::var("LANG").is_err() { c.env("LANG", "C.UTF-8"); }
        if env::var("LC_ALL").is_err() { c.env("LC_ALL", "C.UTF-8"); }
        c.stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
    }
    Ok(())
}
