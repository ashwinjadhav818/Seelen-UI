pub mod constants;
pub mod virtual_desktop;
pub mod rect;

use std::time::Duration;

use tauri::{path::BaseDirectory, AppHandle, Manager, Wry};
use tauri_plugin_shell::ShellExt;
use windows::Win32::Foundation::RECT;

use crate::error_handler::Result;

pub fn sleep_millis(millis: u64) {
    std::thread::sleep(Duration::from_millis(millis));
}

pub fn filename_from_path(path: &str) -> String {
    path.split('\\').last().unwrap_or_default().to_string()
}

pub fn are_overlaped(a: &RECT, b: &RECT) -> bool {
    if a.right < b.left || a.left > b.right || a.bottom < b.top || a.top > b.bottom {
        return false;
    }
    true
}

pub fn pascal_to_kebab(input: &str) -> String {
    let mut kebab_case = String::new();
    let mut prev_char_lowercase = false;
    for c in input.chars() {
        if c.is_uppercase() {
            if prev_char_lowercase {
                kebab_case.push('-');
            }
            kebab_case.push(c.to_ascii_lowercase());
            prev_char_lowercase = false;
        } else {
            kebab_case.push(c);
            prev_char_lowercase = true;
        }
    }
    kebab_case
}

pub fn kebab_to_pascal(input: &str) -> String {
    let mut pascal_case = String::new();
    let mut prev_char_dash = false;
    for c in input.chars() {
        if c == '-' {
            prev_char_dash = true;
        } else {
            if prev_char_dash || pascal_case.is_empty() {
                pascal_case.push(c.to_ascii_uppercase());
                prev_char_dash = false;
            } else {
                pascal_case.push(c);
            }
        }
    }
    pascal_case
}

pub fn is_windows_10() -> bool {
    matches!(os_info::get().version(), os_info::Version::Semantic(_, _, x) if x >= &10240 && x < &22000)
}

pub fn is_windows_11() -> bool {
    matches!(os_info::get().version(), os_info::Version::Semantic(_, _, x) if x >= &22000)
}

pub fn run_ahk_file(handle: &AppHandle<Wry>, ahk_file: &str) -> Result<()> {
    log::trace!("Starting AHK: {}", ahk_file);

    let ahk_script_path = handle
        .path()
        .resolve(format!("static/{}", ahk_file), BaseDirectory::Resource)?;

    if !ahk_script_path.exists() {
        return Err(format!("AHK script not found: {}", ahk_file).into());
    }

    let ahk_script_path = ahk_script_path
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_owned();

    let ahk_path = handle
        .path()
        .resolve("static/redis/AutoHotkey.exe", BaseDirectory::Resource)?
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_owned();

    handle
        .shell()
        .command(ahk_path)
        .arg(ahk_script_path)
        .spawn()?;

    Ok(())
}
