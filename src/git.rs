use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitFileStatus {
    Modified,
    Staged,
    Untracked,
    Deleted,
    Renamed,
    Ignored,
}

#[derive(Clone, Debug, Default)]
pub struct GitRepoInfo {
    pub branch: Option<String>,
    pub file_statuses: HashMap<PathBuf, GitFileStatus>,
}

pub fn find_git_root(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir.to_path_buf();
    loop {
        let git_dir = current.join(".git");
        if git_dir.exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

pub fn get_git_branch(git_root: &Path) -> Option<String> {
    let head_path = git_root.join(".git").join("HEAD");
    if let Ok(content) = fs::read_to_string(head_path) {
        let trimmed = content.trim();
        if let Some(ref_path) = trimmed.strip_prefix("ref: refs/heads/") {
            return Some(ref_path.to_string());
        } else if trimmed.len() >= 7 {
            // Detached HEAD
            return Some(trimmed[..7].to_string());
        }
    }
    None
}

pub fn get_git_status(dir: &Path) -> Option<GitRepoInfo> {
    let git_root = find_git_root(dir)?;
    let branch = get_git_branch(&git_root);

    let output = Command::new("git")
        .arg("-C")
        .arg(&git_root)
        .args(["status", "--porcelain", "-uall"])
        .output()
        .ok()?;

    if !output.status.success() {
        return Some(GitRepoInfo {
            branch,
            file_statuses: HashMap::new(),
        });
    }

    let status_str = String::from_utf8_lossy(&output.stdout);
    let mut file_statuses = HashMap::new();

    for line in status_str.lines() {
        if line.len() < 4 {
            continue;
        }
        let index_stat = line.as_bytes()[0];
        let work_stat = line.as_bytes()[1];
        let path_str = line[3..].trim().trim_matches('"');
        let full_path = git_root.join(path_str);

        let status = if index_stat == b'?' && work_stat == b'?' {
            GitFileStatus::Untracked
        } else if index_stat == b'A' || index_stat == b'M' || index_stat == b'R' {
            GitFileStatus::Staged
        } else if work_stat == b'M' {
            GitFileStatus::Modified
        } else if work_stat == b'D' || index_stat == b'D' {
            GitFileStatus::Deleted
        } else {
            GitFileStatus::Modified
        };

        file_statuses.insert(full_path, status);
    }

    Some(GitRepoInfo {
        branch,
        file_statuses,
    })
}
