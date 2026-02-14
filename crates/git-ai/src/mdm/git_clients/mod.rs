pub mod fork_app;
#[cfg(target_os = "macos")]
pub mod mac_prefs;
pub mod sublime_merge;

pub use installers_git_clients::mdm::git_clients::get_all_git_client_installers;
