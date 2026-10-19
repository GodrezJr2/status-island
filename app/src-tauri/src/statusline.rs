use std::{
    io::{Read, Write},
    os::windows::process::CommandExt,
    process::{Command, Stdio},
};

pub fn run() -> i32 {
    let mut input = Vec::new();
    if std::io::stdin().read_to_end(&mut input).is_err() {
        return 1;
    }
    if let Ok(v) = serde_json::from_slice(&input) {
        crate::pipe::send("statusline", &v);
    }
    let Some(home) = std::env::var_os("LOCALAPPDATA") else {
        return 0;
    };
    let path = std::path::PathBuf::from(home)
        .join("StatusIsland")
        .join("previous-statusline.txt");
    let Ok(command) = std::fs::read_to_string(path) else {
        return 0;
    };
    if command.trim().is_empty() {
        return 0;
    }
    let Ok(mut child) = Command::new("cmd")
        .creation_flags(0x0800_0000) // CREATE_NO_WINDOW: no console flash from a GUI program
        .args(["/S", "/C"])
        .raw_arg(format!("\"{command}\""))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return 0;
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(&input);
    }
    if let Ok(output) = child.wait_with_output() {
        let _ = std::io::stdout().write_all(&output.stdout);
    }
    0
}
