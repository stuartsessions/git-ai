use crate::error::GitAiError;
use std::io::IsTerminal;
use std::path::PathBuf;

static DEBUG_ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
static DEBUG_PERFORMANCE_LEVEL: std::sync::OnceLock<u8> = std::sync::OnceLock::new();
static IS_TERMINAL: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

fn is_debug_enabled() -> bool {
    *DEBUG_ENABLED.get_or_init(|| {
        (cfg!(debug_assertions)
            || std::env::var("GIT_AI_DEBUG").unwrap_or_default() == "1"
            || std::env::var("GIT_AI_DEBUG_PERFORMANCE").unwrap_or_default() != "")
            && std::env::var("GIT_AI_DEBUG").unwrap_or_default() != "0"
    })
}

fn is_debug_performance_enabled() -> bool {
    debug_performance_level() >= 1
}

fn debug_performance_level() -> u8 {
    *DEBUG_PERFORMANCE_LEVEL.get_or_init(|| {
        std::env::var("GIT_AI_DEBUG_PERFORMANCE")
            .unwrap_or_default()
            .parse::<u8>()
            .unwrap_or(0)
    })
}

pub fn debug_performance_log(msg: &str) {
    if is_debug_performance_enabled() {
        eprintln!("\x1b[1;33m[git-ai (perf)]\x1b[0m {}", msg);
    }
}

pub fn debug_performance_log_structured(json: serde_json::Value) {
    if debug_performance_level() >= 2 {
        eprintln!("\x1b[1;33m[git-ai (perf-json)]\x1b[0m {}", json);
    }
}

pub fn debug_log(msg: &str) {
    if is_debug_enabled() {
        eprintln!("\x1b[1;33m[git-ai]\x1b[0m {}", msg);
    }
}

#[inline]
pub fn normalize_to_posix(path: &str) -> String {
    path.replace('\\', "/")
}

pub fn current_git_ai_exe() -> Result<PathBuf, GitAiError> {
    let path = std::env::current_exe()?;

    let git_name = if cfg!(windows) { "git.exe" } else { "git" };
    let git_ai_name = if cfg!(windows) { "git-ai.exe" } else { "git-ai" };

    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) && file_name == git_name {
        let git_ai_path = path.with_file_name(git_ai_name);
        if git_ai_path.exists() {
            return Ok(git_ai_path);
        }
        return Ok(PathBuf::from(git_ai_name));
    }

    Ok(path)
}

pub fn is_interactive_terminal() -> bool {
    *IS_TERMINAL.get_or_init(|| std::io::stdin().is_terminal())
}

pub struct LockFile {
    _file: std::fs::File,
}

impl LockFile {
    pub fn try_acquire(path: &std::path::Path) -> Option<Self> {
        let file = try_lock_exclusive(path)?;
        Some(Self { _file: file })
    }
}

#[cfg(unix)]
#[allow(clippy::suspicious_open_options)]
fn try_lock_exclusive(path: &std::path::Path) -> Option<std::fs::File> {
    use std::os::unix::io::AsRawFd;
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .ok()?;
    let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if rc != 0 {
        return None;
    }
    Some(file)
}

#[cfg(windows)]
#[allow(clippy::suspicious_open_options)]
fn try_lock_exclusive(path: &std::path::Path) -> Option<std::fs::File> {
    use std::os::windows::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .share_mode(0)
        .open(path)
        .ok()
}

#[cfg(windows)]
pub const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn unescape_git_path(path: &str) -> String {
    if !path.starts_with('"') || !path.ends_with('"') {
        return path.to_string();
    }

    let inner = &path[1..path.len() - 1];

    let mut bytes: Vec<u8> = Vec::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek() {
                Some('\\') => {
                    chars.next();
                    bytes.push(b'\\');
                }
                Some('"') => {
                    chars.next();
                    bytes.push(b'"');
                }
                Some('n') => {
                    chars.next();
                    bytes.push(b'\n');
                }
                Some('t') => {
                    chars.next();
                    bytes.push(b'\t');
                }
                Some('r') => {
                    chars.next();
                    bytes.push(b'\r');
                }
                Some(d) if d.is_ascii_digit() => {
                    let mut octal = String::new();
                    for _ in 0..3 {
                        if let Some(&d) = chars.peek() {
                            if d.is_ascii_digit() && d <= '7' {
                                octal.push(chars.next().unwrap());
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    if let Ok(byte_val) = u8::from_str_radix(&octal, 8) {
                        bytes.push(byte_val);
                    }
                }
                _ => {
                    bytes.push(b'\\');
                }
            }
        } else {
            let mut buf = [0u8; 4];
            let encoded = c.encode_utf8(&mut buf);
            bytes.extend_from_slice(encoded.as_bytes());
        }
    }

    String::from_utf8(bytes).unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockfile_acquire_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let lock_path = dir.path().join("test.lock");
        let lock = LockFile::try_acquire(&lock_path);
        assert!(lock.is_some(), "should acquire lock on a fresh path");
    }

    #[test]
    fn test_lockfile_second_acquire_blocked() {
        let dir = tempfile::tempdir().unwrap();
        let lock_path = dir.path().join("test.lock");
        let _first = LockFile::try_acquire(&lock_path).expect("first acquire should succeed");
        let second = LockFile::try_acquire(&lock_path);
        assert!(second.is_none(), "second acquire should be blocked");
    }

    #[test]
    fn test_lockfile_released_on_drop() {
        let dir = tempfile::tempdir().unwrap();
        let lock_path = dir.path().join("test.lock");
        {
            let _lock = LockFile::try_acquire(&lock_path).expect("first acquire should succeed");
        }
        let second = LockFile::try_acquire(&lock_path);
        assert!(second.is_some(), "should acquire lock after previous holder is dropped");
    }

    #[test]
    fn test_unescape_git_path_simple() {
        assert_eq!(unescape_git_path("simple.txt"), "simple.txt");
        assert_eq!(unescape_git_path("path/to/file.rs"), "path/to/file.rs");
    }

    #[test]
    fn test_unescape_git_path_quoted_with_spaces() {
        assert_eq!(unescape_git_path("\"path with spaces.txt\""), "path with spaces.txt");
        assert_eq!(unescape_git_path("\"dir name/file name.txt\""), "dir name/file name.txt");
    }

    #[test]
    fn test_unescape_git_path_chinese_characters() {
        assert_eq!(unescape_git_path("\"\\344\\270\\255\\346\\226\\207.txt\""), "中文.txt");
    }
}
