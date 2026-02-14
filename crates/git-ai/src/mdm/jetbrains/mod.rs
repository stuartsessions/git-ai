pub mod detection;
pub mod download;
pub mod ide_types;

#[allow(unused_imports)]
pub use installers_agents::mdm::jetbrains::{
    DetectedIde, MARKETPLACE_URL, MIN_INTELLIJ_BUILD, PLUGIN_ID,
    download_plugin_from_marketplace, find_jetbrains_installations,
    install_plugin_to_directory, install_plugin_via_cli, is_plugin_installed,
};
