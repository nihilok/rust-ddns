use std::process;

#[allow(dead_code)]
pub fn parse_interval_secs(s: &str) -> u64 {
    if let Some(mins) = s.strip_suffix("min") {
        if let Ok(n) = mins.parse::<u64>() {
            return n * 60;
        }
    }
    if let Some(hours) = s.strip_suffix("h") {
        if let Ok(n) = hours.parse::<u64>() {
            return n * 3600;
        }
    }
    s.parse::<u64>().unwrap_or(300)
}

#[cfg(target_os = "linux")]
pub fn install(interval: &str, log_file: Option<&str>, config_file: Option<&str>) {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let home = std::env::var("HOME").unwrap_or_else(|_| {
        eprintln!("ERROR: HOME environment variable is not set");
        process::exit(1);
    });

    let bin_dir = format!("{}/.local/bin", home);
    fs::create_dir_all(&bin_dir).unwrap_or_else(|e| {
        eprintln!("ERROR: Could not create {}: {}", bin_dir, e);
        process::exit(1);
    });

    let exe = std::env::current_exe().unwrap_or_else(|e| {
        eprintln!("ERROR: Could not determine current exe path: {}", e);
        process::exit(1);
    });

    let binary_dest = format!("{}/rust-ddns", bin_dir);
    fs::copy(&exe, &binary_dest).unwrap_or_else(|e| {
        eprintln!("ERROR: Could not copy binary to {}: {}", binary_dest, e);
        process::exit(1);
    });

    let log_path = log_file
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}/.rust-ddns.log", home));

    let config_arg = if let Some(cf) = config_file {
        format!(" --config-file {}", cf)
    } else {
        String::new()
    };

    let wrapper_path = format!("{}/ddnsd-rust-ddns", bin_dir);
    let wrapper_content = format!("#!/bin/bash\n\
RUST_DDNS_LOG_FILE={log}\n\
touch \"$RUST_DDNS_LOG_FILE\"\n\
cd $HOME || exit 1\n\
rust-ddns{config} &>> \"$RUST_DDNS_LOG_FILE\"\n\
echo \"$(tail -n 200 \"$RUST_DDNS_LOG_FILE\")\" > \"$RUST_DDNS_LOG_FILE\"\n",
        log = log_path,
        config = config_arg,
    );

    fs::write(&wrapper_path, wrapper_content).unwrap_or_else(|e| {
        eprintln!("ERROR: Could not write wrapper script: {}", e);
        process::exit(1);
    });
    fs::set_permissions(&wrapper_path, fs::Permissions::from_mode(0o755)).unwrap_or_else(|e| {
        eprintln!("ERROR: Could not set wrapper permissions: {}", e);
        process::exit(1);
    });

    let timer_content = format!("[Unit]\n\
Description=Runs rust-ddns every {interval}\n\
\n\
[Timer]\n\
OnBootSec={interval}\n\
OnUnitActiveSec={interval}\n\
Unit=rust-ddns.service\n\
\n\
[Install]\n\
WantedBy=multi-user.target\n");

    let service_content = format!("[Unit]\n\
Description=Run rust-ddns once\n\
\n\
[Service]\n\
User={user}\n\
WorkingDir={home}\n\
ExecStart={wrapper}\n\
Environment=HOME={home}\n\
Environment=PATH={bin_dir}:/usr/local/bin:/usr/bin:/bin\n",
        user = std::env::var("USER").unwrap_or_else(|_| "nobody".to_string()),
        home = home,
        wrapper = wrapper_path,
        bin_dir = bin_dir,
    );

    write_file_as_root("/etc/systemd/system/rust-ddns.timer", &timer_content);
    write_file_as_root("/etc/systemd/system/rust-ddns.service", &service_content);

    run_sudo(&["systemctl", "daemon-reload"]);

    println!("Installation complete!");
    println!("Run: sudo systemctl enable --now rust-ddns.timer");
}

#[cfg(target_os = "linux")]
pub fn uninstall(purge: bool) {
    use std::fs;

    let home = std::env::var("HOME").unwrap_or_default();
    let bin_dir = format!("{}/.local/bin", home);

    let _ = process::Command::new("sudo")
        .args(["systemctl", "stop", "rust-ddns.timer"])
        .status();
    let _ = process::Command::new("sudo")
        .args(["systemctl", "disable", "rust-ddns.timer"])
        .status();
    let _ = process::Command::new("sudo")
        .args(["systemctl", "stop", "rust-ddns.service"])
        .status();
    let _ = process::Command::new("sudo")
        .args(["systemctl", "disable", "rust-ddns.service"])
        .status();

    let _ = process::Command::new("sudo")
        .args(["rm", "-f", "/etc/systemd/system/rust-ddns.timer"])
        .status();
    let _ = process::Command::new("sudo")
        .args(["rm", "-f", "/etc/systemd/system/rust-ddns.service"])
        .status();

    run_sudo(&["systemctl", "daemon-reload"]);

    let binary = format!("{}/rust-ddns", bin_dir);
    let wrapper = format!("{}/ddnsd-rust-ddns", bin_dir);
    let _ = fs::remove_file(&binary);
    let _ = fs::remove_file(&wrapper);

    if purge {
        let conf = format!("{}/.ddns.conf", home);
        let log = format!("{}/.rust-ddns.log", home);
        let _ = fs::remove_file(&conf);
        let _ = fs::remove_file(&log);
        println!("Purged config and log files.");
    }

    println!("Uninstallation complete!");
}

#[cfg(target_os = "macos")]
pub fn install(interval: &str, log_file: Option<&str>, config_file: Option<&str>) {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let home = std::env::var("HOME").unwrap_or_else(|_| {
        eprintln!("ERROR: HOME environment variable is not set");
        process::exit(1);
    });

    let bin_dir = format!("{}/.local/bin", home);
    fs::create_dir_all(&bin_dir).unwrap_or_else(|e| {
        eprintln!("ERROR: Could not create {}: {}", bin_dir, e);
        process::exit(1);
    });

    let exe = std::env::current_exe().unwrap_or_else(|e| {
        eprintln!("ERROR: Could not determine current exe path: {}", e);
        process::exit(1);
    });

    let binary_dest = format!("{}/rust-ddns", bin_dir);
    fs::copy(&exe, &binary_dest).unwrap_or_else(|e| {
        eprintln!("ERROR: Could not copy binary: {}", e);
        process::exit(1);
    });
    fs::set_permissions(&binary_dest, fs::Permissions::from_mode(0o755)).unwrap_or_else(|e| {
        eprintln!("ERROR: Could not set binary permissions: {}", e);
        process::exit(1);
    });

    let interval_secs = parse_interval_secs(interval);
    let log_path = log_file
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}/.rust-ddns.log", home));

    let config_arg = if let Some(cf) = config_file {
        format!(" --config-file {}", cf)
    } else {
        String::new()
    };

    let wrapper_path = format!("{}/ddnsd-rust-ddns", bin_dir);
    let wrapper_content = format!("#!/bin/bash\n\
RUST_DDNS_LOG_FILE={log}\n\
touch \"$RUST_DDNS_LOG_FILE\"\n\
cd $HOME || exit 1\n\
rust-ddns{config} &>> \"$RUST_DDNS_LOG_FILE\"\n\
echo \"$(tail -n 200 \"$RUST_DDNS_LOG_FILE\")\" > \"$RUST_DDNS_LOG_FILE\"\n",
        log = log_path,
        config = config_arg,
    );

    fs::write(&wrapper_path, wrapper_content).unwrap_or_else(|e| {
        eprintln!("ERROR: Could not write wrapper script: {}", e);
        process::exit(1);
    });
    fs::set_permissions(&wrapper_path, fs::Permissions::from_mode(0o755)).unwrap_or_else(|e| {
        eprintln!("ERROR: Could not set wrapper permissions: {}", e);
        process::exit(1);
    });

    let plist_dir = format!("{}/Library/LaunchAgents", home);
    fs::create_dir_all(&plist_dir).unwrap_or_else(|e| {
        eprintln!("ERROR: Could not create {}: {}", plist_dir, e);
        process::exit(1);
    });

    let plist_path = format!("{}/com.rust-ddns.plist", plist_dir);
    let plist_content = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n\
<dict>\n\
    <key>Label</key>\n\
    <string>com.rust-ddns</string>\n\
    <key>ProgramArguments</key>\n\
    <array>\n\
        <string>{wrapper}</string>\n\
    </array>\n\
    <key>StartInterval</key>\n\
    <integer>{secs}</integer>\n\
    <key>RunAtLoad</key>\n\
    <true/>\n\
    <key>StandardOutPath</key>\n\
    <string>{log}</string>\n\
    <key>StandardErrorPath</key>\n\
    <string>{log}</string>\n\
</dict>\n\
</plist>\n",
        wrapper = wrapper_path,
        secs = interval_secs,
        log = log_path,
    );

    fs::write(&plist_path, plist_content).unwrap_or_else(|e| {
        eprintln!("ERROR: Could not write plist: {}", e);
        process::exit(1);
    });

    let uid = unsafe { libc::getuid() };
    let status = process::Command::new("launchctl")
        .args(["bootstrap", &format!("gui/{}", uid), &plist_path])
        .status()
        .unwrap_or_else(|e| {
            eprintln!("ERROR: launchctl failed: {}", e);
            process::exit(1);
        });
    if !status.success() {
        eprintln!("ERROR: launchctl bootstrap failed");
        process::exit(1);
    }

    println!("Installation complete!");
}

#[cfg(target_os = "macos")]
pub fn uninstall(purge: bool) {
    use std::fs;

    let home = std::env::var("HOME").unwrap_or_default();
    let plist_path = format!("{}/Library/LaunchAgents/com.rust-ddns.plist", home);
    let uid = unsafe { libc::getuid() };

    let _ = process::Command::new("launchctl")
        .args(["bootout", &format!("gui/{}", uid), &plist_path])
        .status();

    let _ = fs::remove_file(&plist_path);

    let bin_dir = format!("{}/.local/bin", home);
    let _ = fs::remove_file(format!("{}/rust-ddns", bin_dir));
    let _ = fs::remove_file(format!("{}/ddnsd-rust-ddns", bin_dir));

    if purge {
        let _ = fs::remove_file(format!("{}/.ddns.conf", home));
        let _ = fs::remove_file(format!("{}/.rust-ddns.log", home));
        println!("Purged config and log files.");
    }

    println!("Uninstallation complete!");
}

#[cfg(target_os = "windows")]
pub fn install(interval: &str, log_file: Option<&str>, config_file: Option<&str>) {
    use std::fs;

    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| {
        eprintln!("ERROR: LOCALAPPDATA is not set");
        process::exit(1);
    });

    let install_dir = format!("{}\\rust-ddns", local_app_data);
    fs::create_dir_all(&install_dir).unwrap_or_else(|e| {
        eprintln!("ERROR: Could not create {}: {}", install_dir, e);
        process::exit(1);
    });

    let exe = std::env::current_exe().unwrap_or_else(|e| {
        eprintln!("ERROR: Could not determine current exe: {}", e);
        process::exit(1);
    });

    let binary_dest = format!("{}\\rust-ddns.exe", install_dir);
    fs::copy(&exe, &binary_dest).unwrap_or_else(|e| {
        eprintln!("ERROR: Could not copy binary: {}", e);
        process::exit(1);
    });

    let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
    let log_path = log_file
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}\\.rust-ddns.log", user_profile));

    let config_arg = if let Some(cf) = config_file {
        format!(" --config-file {}", cf)
    } else {
        String::new()
    };

    let wrapper_dest = format!("{}\\ddnsd.cmd", install_dir);
    let wrapper_content = format!(
        "@echo off\r\n\
set LOG_FILE={log}\r\n\
if not exist \"%LOG_FILE%\" type nul > \"%LOG_FILE%\"\r\n\
\"{binary}\"{config} >> \"%LOG_FILE%\" 2>&1\r\n\
powershell -Command \"Get-Content '%LOG_FILE%' -Tail 200 | Set-Content '%LOG_FILE%'\"\r\n",
        log = log_path,
        binary = binary_dest,
        config = config_arg,
    );
    fs::write(&wrapper_dest, &wrapper_content).unwrap_or_else(|e| {
        eprintln!("ERROR: Could not write wrapper: {}", e);
        process::exit(1);
    });

    let interval_secs = parse_interval_secs(interval);
    let interval_mins = std::cmp::max(1, interval_secs / 60);

    let status = process::Command::new("schtasks")
        .args([
            "/Create", "/F",
            "/TN", "rust-ddns",
            "/TR", &format!("\"{}\"", wrapper_dest),
            "/SC", "MINUTE",
            "/MO", &interval_mins.to_string(),
        ])
        .status()
        .unwrap_or_else(|e| {
            eprintln!("ERROR: schtasks failed: {}", e);
            process::exit(1);
        });

    if !status.success() {
        eprintln!("ERROR: Failed to register scheduled task");
        process::exit(1);
    }

    println!("Installation complete!");
}

#[cfg(target_os = "windows")]
pub fn uninstall(purge: bool) {
    use std::fs;

    let _ = process::Command::new("schtasks")
        .args(["/Delete", "/F", "/TN", "rust-ddns"])
        .status();

    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let install_dir = format!("{}\\rust-ddns", local_app_data);
    let _ = fs::remove_dir_all(&install_dir);

    if purge {
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        let _ = fs::remove_file(format!("{}\\.ddns.conf", user_profile));
        let _ = fs::remove_file(format!("{}\\.rust-ddns.log", user_profile));
        println!("Purged config and log files.");
    }

    println!("Uninstallation complete!");
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn install(_interval: &str, _log_file: Option<&str>, _config_file: Option<&str>) {
    eprintln!("ERROR: install subcommand is not supported on this platform.");
    process::exit(1);
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn uninstall(_purge: bool) {
    eprintln!("ERROR: uninstall subcommand is not supported on this platform.");
    process::exit(1);
}

#[cfg(target_os = "linux")]
fn write_file_as_root(path: &str, content: &str) {
    use std::io::Write;
    let mut child = process::Command::new("sudo")
        .args(["tee", path])
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::null())
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("ERROR: Could not run sudo tee {}: {}", path, e);
            process::exit(1);
        });
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(content.as_bytes()).unwrap_or_else(|e| {
            eprintln!("ERROR: Could not write to sudo tee stdin: {}", e);
            process::exit(1);
        });
    }
    let status = child.wait().unwrap_or_else(|e| {
        eprintln!("ERROR: sudo tee failed: {}", e);
        process::exit(1);
    });
    if !status.success() {
        eprintln!("ERROR: sudo tee {} failed with status {}", path, status);
        process::exit(1);
    }
}

#[cfg(target_os = "linux")]
fn run_sudo(args: &[&str]) {
    let status = process::Command::new("sudo")
        .args(args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("ERROR: sudo {:?} failed: {}", args, e);
            process::exit(1);
        });
    if !status.success() {
        eprintln!("ERROR: sudo {:?} failed with status {}", args, status);
        process::exit(1);
    }
}
