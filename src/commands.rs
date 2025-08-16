use std::io::Result;
use std::process::Command;

/// Spawn a non-blocking shell command using `sh -lc`.
pub fn run_shell(cmd: &str) -> Result<()> {
    #[cfg(windows)]
    {
        run_windows_command(cmd)?;
    }
    #[cfg(not(windows))]
    {
        use std::env;
        use std::process::Stdio;
        let mut c = Command::new("sh");
        c.arg("-lc").arg(cmd);
        if env::var("LANG").is_err() { c.env("LANG", "C.UTF-8"); }
        if env::var("LC_ALL").is_err() { c.env("LC_ALL", "C.UTF-8"); }
        c.stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
    }
    Ok(())
}

#[cfg(windows)]
pub fn run_windows_command(cmd: &str) -> Result<()> {
    use std::os::windows::process::CommandExt;
    use winapi::um::winbase::CREATE_NO_WINDOW;
    
    // Handle special Windows targets
    if cmd.starts_with("shell:") {
        // Shell folders - use explorer
        let mut command = Command::new("explorer.exe");
        command.arg(cmd);
        command.creation_flags(CREATE_NO_WINDOW);
        command.spawn()?;
    } else if cmd.starts_with("ms-settings:") {
        // Windows 10/11 Settings - use start
        let mut command = Command::new("cmd");
        command.args(&["/C", "start", "", cmd]);
        command.creation_flags(CREATE_NO_WINDOW);
        command.spawn()?;
    } else if cmd.starts_with("ms-") {
        // Other MS protocols - use rundll32
        let mut command = Command::new("rundll32.exe");
        command.args(&["url.dll,FileProtocolHandler", cmd]);
        command.creation_flags(CREATE_NO_WINDOW);
        command.spawn()?;
    } else if cmd.contains("::") && cmd.len() < 100 {
        // CLSID shortcuts - use explorer
        let mut command = Command::new("explorer.exe");
        command.arg(cmd);
        command.creation_flags(CREATE_NO_WINDOW);
        command.spawn()?;
    } else if cmd.starts_with("mmc.exe") {
        // Management console - run directly
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let mut command = Command::new(parts[0]);
        if parts.len() > 1 {
            command.arg(parts[1].trim_matches('"'));
        }
        command.creation_flags(CREATE_NO_WINDOW);
        command.spawn()?;
    } else if cmd.starts_with("rundll32.exe") {
        // Direct rundll32 call
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let mut command = Command::new("rundll32.exe");
        if parts.len() > 1 {
            // Split the rundll32 arguments
            let args: Vec<&str> = parts[1].split(',').collect();
            for arg in args {
                command.arg(arg.trim());
            }
        }
        command.creation_flags(CREATE_NO_WINDOW);
        command.spawn()?;
    } else if cmd.contains(' ') && cmd.starts_with('"') {
        // Quoted executable with arguments
        let parts: Vec<&str> = cmd.splitn(2, "\" ").collect();
        if parts.len() == 2 {
            let exe = parts[0].trim_start_matches('"');
            let args = parts[1];
            let mut command = Command::new(exe);
            for arg in args.split_whitespace() {
                command.arg(arg.trim_matches('"'));
            }
            command.creation_flags(CREATE_NO_WINDOW);
            command.spawn()?;
        } else {
            // Single quoted path
            let mut command = Command::new(cmd.trim_matches('"'));
            command.creation_flags(CREATE_NO_WINDOW);
            command.spawn()?;
        }
    } else if cmd.contains(' ') && !cmd.starts_with('"') {
        // Unquoted executable with arguments
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let mut command = Command::new(parts[0]);
        if parts.len() > 1 {
            for arg in parts[1].split_whitespace() {
                command.arg(arg.trim_matches('"'));
            }
        }
        command.creation_flags(CREATE_NO_WINDOW);
        command.spawn()?;
    } else {
        // Simple executable name or path
        let mut command = Command::new(cmd.trim_matches('"'));
        command.creation_flags(CREATE_NO_WINDOW);
        command.spawn()?;
    }
    
    Ok(())
}

#[cfg(windows)]
pub fn run_windows_command_hidden(program: &str, args: &[&str]) -> Result<()> {
    use std::os::windows::process::CommandExt;
    use winapi::um::winbase::CREATE_NO_WINDOW;
    
    let mut cmd = Command::new(program);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.spawn()?;
    Ok(())
}
