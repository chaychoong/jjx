use jj_cli::config::{resolved_config_values, AnnotatedValue};
use jj_lib::config::{ConfigNamePathBuf, ConfigSource};
use jj_lib::settings::UserSettings;

pub fn resolve_configs(settings: &UserSettings) -> Vec<AnnotatedValue> {
    let mut annotated_values =
        resolved_config_values(&settings.config(), &ConfigNamePathBuf::root());
    annotated_values.retain(|annotated| !annotated.is_overridden);
    annotated_values.retain(|annotated| annotated.source != ConfigSource::Default);
    annotated_values
}

pub fn configs_to_tuple_list(configs: Vec<AnnotatedValue>) -> Vec<(String, String)> {
    configs
        .into_iter()
        .map(|annotated| {
            (
                annotated.name.to_string(),
                annotated.value.clone().decorated("", "").to_string(),
            )
        })
        .collect()
}
