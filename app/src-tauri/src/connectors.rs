use jsonc_parser::{parse_to_serde_value, ParseOptions};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    io::Write,
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use windows::{
    core::PCWSTR,
    Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectorState {
    pub wrapper_set: bool,
    pub statusline_set: bool,
    pub backups: Vec<PathBuf>,
    pub vs_path: PathBuf,
    pub claude_path: PathBuf,
    pub data_dir: PathBuf,
    pub vs_existed: bool,
    pub claude_existed: bool,
    pub installed_vs_len: usize,
    pub installed_vs_hash: u64,
    pub installed_claude_len: usize,
    pub installed_claude_hash: u64,
}

pub fn enable_claude(app_exe: &Path) -> Result<ConnectorState, String> {
    if !app_exe.is_file() || !app_exe.with_file_name("si-wrap.exe").is_file() {
        return Err("installed wrapper executable is missing".into());
    }
    let home = std::env::var_os("USERPROFILE").ok_or("user home unavailable")?;
    let roaming = std::env::var_os("APPDATA").ok_or("roaming AppData unavailable")?;
    let local = std::env::var_os("LOCALAPPDATA").ok_or("local AppData unavailable")?;
    enable_claude_at(
        app_exe,
        &PathBuf::from(roaming).join("Code/User/settings.json"),
        &PathBuf::from(home).join(".claude/settings.json"),
        &PathBuf::from(local).join("StatusIsland"),
    )
}

pub fn disable_claude() -> Result<(), String> {
    let local = std::env::var_os("LOCALAPPDATA").ok_or("local AppData unavailable")?;
    let manifest = PathBuf::from(local).join("StatusIsland/connector-state.json");
    let state: ConnectorState =
        serde_json::from_slice(&fs::read(manifest).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    disable_claude_at(&state)
}

pub fn enable_claude_at(
    app_exe: &Path,
    vs: &Path,
    claude: &Path,
    data_dir: &Path,
) -> Result<ConnectorState, String> {
    if data_dir.join("connector-state.json").exists() {
        return Err("Claude connector is already enabled".into());
    }
    let vs_existed = vs.exists();
    let claude_existed = claude.exists();
    let vs_original = if vs_existed {
        fs::read(vs).map_err(|e| e.to_string())?
    } else {
        b"{}\n".to_vec()
    };
    let claude_original = if claude_existed {
        fs::read(claude).map_err(|e| e.to_string())?
    } else {
        b"{}\n".to_vec()
    };
    let vs_text = std::str::from_utf8(&vs_original).map_err(|e| e.to_string())?;
    let claude_text = std::str::from_utf8(&claude_original).map_err(|e| e.to_string())?;
    let vs_json = parse_jsonc(vs_text)?;
    let claude_json = parse_jsonc(claude_text)?;
    let wrapper = app_exe
        .with_file_name("si-wrap.exe")
        .to_string_lossy()
        .to_string();
    let mut changed_vs = crate::jsonc_edit::set_key(
        vs_text,
        "claudeCode.claudeProcessWrapper",
        Some(&Value::String(wrapper)),
    )?;
    if vs_json.get("claudeCode.initialPermissionMode").is_none() {
        if let Some(mode) = claude_json
            .get("permissions")
            .and_then(|p| p.get("defaultMode"))
        {
            changed_vs = crate::jsonc_edit::set_key(
                &changed_vs,
                "claudeCode.initialPermissionMode",
                Some(mode),
            )?;
        }
    }
    let previous = claude_json
        .get("statusLine")
        .and_then(|s| s.get("command"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let command = format!("\"{}\" --statusline", app_exe.display());
    let status_line = serde_json::json!({"type":"command","command":command});
    let changed_claude = crate::jsonc_edit::set_key(claude_text, "statusLine", Some(&status_line))?;

    let backup_dir = data_dir.join("backups");
    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let vs_backup = backup_dir.join(format!("vscode-settings-{stamp}.bak"));
    let claude_backup = backup_dir.join(format!("claude-settings-{stamp}.bak"));
    write_synced(&vs_backup, &vs_original)?;
    write_synced(&claude_backup, &claude_original)?;
    write_atomic(
        &data_dir.join("previous-statusline.txt"),
        previous.as_bytes(),
    )?;

    write_atomic(vs, changed_vs.as_bytes())?;
    if let Err(e) = write_atomic(claude, changed_claude.as_bytes()) {
        let _ = restore_exact(vs, &vs_original, vs_existed);
        return Err(e);
    }
    let state = ConnectorState {
        wrapper_set: true,
        statusline_set: true,
        backups: vec![vs_backup, claude_backup],
        vs_path: vs.to_owned(),
        claude_path: claude.to_owned(),
        data_dir: data_dir.to_owned(),
        vs_existed,
        claude_existed,
        installed_vs_len: changed_vs.len(),
        installed_vs_hash: checksum(changed_vs.as_bytes()),
        installed_claude_len: changed_claude.len(),
        installed_claude_hash: checksum(changed_claude.as_bytes()),
    };
    let manifest = serde_json::to_vec(&state).map_err(|e| e.to_string())?;
    if let Err(e) = write_atomic(&data_dir.join("connector-state.json"), &manifest) {
        let _ = restore_exact(vs, &vs_original, vs_existed);
        let _ = restore_exact(claude, &claude_original, claude_existed);
        return Err(e);
    }
    Ok(state)
}

pub fn disable_claude_at(state: &ConnectorState) -> Result<(), String> {
    let vs_original = fs::read(state.backups.first().ok_or("VS Code backup missing")?)
        .map_err(|e| e.to_string())?;
    let claude_original = fs::read(state.backups.get(1).ok_or("Claude backup missing")?)
        .map_err(|e| e.to_string())?;
    restore_file(
        &state.vs_path,
        &vs_original,
        state.vs_existed,
        state.installed_vs_len,
        state.installed_vs_hash,
        &[
            "claudeCode.claudeProcessWrapper",
            "claudeCode.initialPermissionMode",
        ],
    )?;
    restore_file(
        &state.claude_path,
        &claude_original,
        state.claude_existed,
        state.installed_claude_len,
        state.installed_claude_hash,
        &["statusLine"],
    )?;
    let _ = fs::remove_file(state.data_dir.join("previous-statusline.txt"));
    let _ = fs::remove_file(state.data_dir.join("connector-state.json"));
    Ok(())
}

fn restore_file(
    path: &Path,
    original: &[u8],
    existed: bool,
    installed_len: usize,
    installed_hash: u64,
    keys: &[&str],
) -> Result<(), String> {
    let current = fs::read(path).map_err(|e| e.to_string())?;
    if current.len() == installed_len && checksum(&current) == installed_hash {
        return restore_exact(path, original, existed);
    }
    let old = parse_jsonc(std::str::from_utf8(original).map_err(|e| e.to_string())?)?;
    let mut text = String::from_utf8(current).map_err(|e| e.to_string())?;
    for key in keys {
        text = crate::jsonc_edit::set_key(&text, key, old.get(*key))?;
    }
    write_atomic(path, text.as_bytes())
}

fn restore_exact(path: &Path, original: &[u8], existed: bool) -> Result<(), String> {
    if existed {
        write_atomic(path, original)
    } else if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())
    } else {
        Ok(())
    }
}

fn parse_jsonc(text: &str) -> Result<Value, String> {
    parse_to_serde_value::<Value>(text, &ParseOptions::default()).map_err(|e| e.to_string())
}

fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

fn write_synced(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let dir = path.parent().ok_or("file parent unavailable")?;
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let temp = dir.join(format!(".status-island-{stamp}.tmp"));
    write_synced(&temp, bytes)?;
    let from: Vec<u16> = temp
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let to: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let result = unsafe {
        MoveFileExW(
            PCWSTR(from.as_ptr()),
            PCWSTR(to.as_ptr()),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if let Err(e) = result {
        let _ = fs::remove_file(temp);
        return Err(e.to_string());
    }
    Ok(())
}
