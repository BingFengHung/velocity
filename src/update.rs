use serde::Deserialize;
use std::env;
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::Path;
use zip::ZipArchive;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO_OWNER: &str = "BingFengHung";
const REPO_NAME: &str = "velocity";

#[derive(Deserialize, Debug)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize, Debug)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<ReleaseAsset>,
}

pub fn check_and_update(check_only: bool) -> Result<(), String> {
    println!("🔍 正在檢查最新版本...");
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        REPO_OWNER, REPO_NAME
    );

    let response = ureq::get(&api_url)
        .set("User-Agent", "velocity-updater")
        .call()
        .map_err(|e| format!("無法連線至 GitHub API: {}", e))?;

    let release: GithubRelease = response
        .into_json()
        .map_err(|e| format!("解析 GitHub API 回應失敗: {}", e))?;

    let latest_tag = release.tag_name.trim_start_matches('v');
    println!("當前版本: v{}", CURRENT_VERSION);
    println!("最新版本: v{}", latest_tag);

    if !is_newer_version(latest_tag, CURRENT_VERSION) {
        println!("✨ 目前已經是最新版本！");
        return Ok(());
    }

    println!("🚀 發現新版本 v{}！", latest_tag);
    if check_only {
        println!("💡 請執行 `velocity --update` 以自動升級至最新版本。");
        return Ok(());
    }

    let target_asset_name = get_target_asset_name();
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == target_asset_name)
        .ok_or_else(|| {
            format!(
                "未在 Release 中找到適用於目前系統的二進制資產: {}",
                target_asset_name
            )
        })?;

    println!("📦 正在下載更新檔: {}...", asset.name);
    let download_resp = ureq::get(&asset.browser_download_url)
        .set("User-Agent", "velocity-updater")
        .call()
        .map_err(|e| format!("下載資產失敗: {}", e))?;

    let mut buffer = Vec::new();
    download_resp
        .into_reader()
        .read_to_end(&mut buffer)
        .map_err(|e| format!("讀取下載資料失敗: {}", e))?;

    println!("⚡ 正在解壓縮並安裝更新...");
    let current_exe = env::current_exe().map_err(|e| format!("無法取得目前執行檔路徑: {}", e))?;

    if target_asset_name.ends_with(".zip") {
        replace_from_zip(&buffer, &current_exe)?;
    } else {
        replace_direct_binary(&buffer, &current_exe)?;
    }

    println!("🎉 成功升級至 Velocity v{}！", latest_tag);
    Ok(())
}

fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse_v =
        |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse::<u32>().ok()).collect() };

    let latest_parts = parse_v(latest);
    let current_parts = parse_v(current);

    latest_parts > current_parts
}

fn get_target_asset_name() -> &'static str {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "velocity-windows-x86_64.zip"
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "velocity-linux-x86_64.tar.gz"
    }

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "velocity-macos-x86_64.tar.gz"
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "velocity-macos-aarch64.tar.gz"
    }

    #[cfg(not(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64")
    )))]
    {
        "unknown"
    }
}

fn replace_from_zip(zip_data: &[u8], current_exe: &Path) -> Result<(), String> {
    let cursor = Cursor::new(zip_data);
    let mut archive = ZipArchive::new(cursor).map_err(|e| format!("無法解壓縮 ZIP: {}", e))?;

    let binary_name = if cfg!(windows) {
        "velocity.exe"
    } else {
        "velocity"
    };

    let mut binary_file = archive
        .by_name(binary_name)
        .map_err(|_| format!("ZIP 壓縮包中未包含 {}", binary_name))?;

    let mut new_bytes = Vec::new();
    binary_file
        .read_to_end(&mut new_bytes)
        .map_err(|e| format!("讀取新二進制內容失敗: {}", e))?;

    replace_binary_bytes(&new_bytes, current_exe)
}

fn replace_direct_binary(bytes: &[u8], current_exe: &Path) -> Result<(), String> {
    replace_binary_bytes(bytes, current_exe)
}

fn replace_binary_bytes(new_bytes: &[u8], current_exe: &Path) -> Result<(), String> {
    let backup_exe = current_exe.with_extension("old");

    // On Windows, moving the running executable allows a new file to be created in its place
    if backup_exe.exists() {
        let _ = fs::remove_file(&backup_exe);
    }

    if let Err(e) = fs::rename(current_exe, &backup_exe) {
        return Err(format!("無法重命名目前執行檔以進行更新: {}", e));
    }

    let mut new_file = match File::create(current_exe) {
        Ok(f) => f,
        Err(e) => {
            // Restore backup on failure
            let _ = fs::rename(&backup_exe, current_exe);
            return Err(format!("無法建立新執行檔: {}", e));
        }
    };

    if let Err(e) = new_file.write_all(new_bytes) {
        let _ = fs::rename(&backup_exe, current_exe);
        return Err(format!("寫入新執行檔失敗: {}", e));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = new_file.metadata() {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(current_exe, perms);
        }
    }

    Ok(())
}
