use std::sync::OnceLock;

static DEBUG_ENABLED: OnceLock<bool> = OnceLock::new();

fn is_debug_enabled() -> bool {
    *DEBUG_ENABLED.get_or_init(|| {
        (cfg!(debug_assertions) || std::env::var("GIT_AI_DEBUG").unwrap_or_default() == "1")
            && std::env::var("GIT_AI_DEBUG").unwrap_or_default() != "0"
    })
}

pub fn debug_log(msg: &str) {
    if is_debug_enabled() {
        eprintln!("\x1b[1;33m[git-ai]\x1b[0m {}", msg);
    }
}
