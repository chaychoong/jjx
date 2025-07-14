use jj_cli::config::{config_from_environment, default_config_layers, ConfigEnv};
use jj_cli::ui::Ui;
use jj_lib::settings::UserSettings;
use rustler::Error;
use std::path::Path;

pub fn get_settings_from_path(path: &Path) -> Result<UserSettings, Error> {
    let mut raw_config = config_from_environment(default_config_layers());
    let mut config_env = ConfigEnv::from_environment(&Ui::null());
    let _ = config_env.reload_user_config(&mut raw_config);
    config_env.reset_repo_path(Path::new(&path));
    let _ = config_env.reload_repo_config(&mut raw_config);

    let config = config_env
        .resolve_config(&raw_config)
        .map_err(|_e| Error::Atom("Failed to resolve config".into()))?;
    let settings = UserSettings::from_config(config)
        .map_err(|_e| Error::Atom("Failed to create settings".into()))?;

    Ok(settings)
}
