use jj_cli::config::{
    config_from_environment, default_config_layers, resolved_config_values, ConfigEnv,
};
use jj_cli::ui::Ui;
use jj_lib::{
    config::{ConfigNamePathBuf, ConfigSource},
    repo::{ReadonlyRepo, Repo, StoreFactories},
    settings::UserSettings,
    workspace::{
        default_working_copy_factories, DefaultWorkspaceLoaderFactory, Workspace,
        WorkspaceLoaderFactory,
    },
};
use rustler::{Error, NifStruct, Resource, ResourceArc};
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, NifStruct)]
#[module = "Jj.Native.Commit"]
pub struct Commit {
    pub change_id: String,
    pub change_id_short_len: u8,
    pub commit_id: String,
    pub commit_id_short_len: u8,
    pub message_first_line: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: i64,
}

pub struct WorkspaceResource(Mutex<Workspace>);

#[rustler::resource_impl]
impl Resource for WorkspaceResource {}

type WorkspaceArc = ResourceArc<WorkspaceResource>;

fn commit_to_erl_commit(repo: &ReadonlyRepo, commit: &jj_lib::commit::Commit) -> Commit {
    let change_id = commit.change_id();
    let change_id_short_len: u8 = repo
        .shortest_unique_change_id_prefix_len(change_id)
        .try_into()
        .unwrap_or(0);
    let commit_id = commit.id();
    let commit_id_short_len: u8 = repo
        .index()
        .shortest_unique_commit_id_prefix_len(commit_id)
        .try_into()
        .unwrap_or(0);
    let commit_description = commit.description();
    let commit_author = commit.author();

    Commit {
        change_id: change_id.to_string(),
        change_id_short_len,
        commit_id: commit_id.to_string(),
        commit_id_short_len,
        message_first_line: commit_description.lines().next().unwrap_or("").to_string(),
        author_name: commit_author.name.clone(),
        author_email: commit_author.email.clone(),
        timestamp: commit_author.timestamp.timestamp.0 as i64,
    }
}

fn get_settings_from_path(path: &Path) -> Result<UserSettings, Error> {
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

#[rustler::nif]
fn get_workspace(path: String) -> Result<WorkspaceArc, Error> {
    let path = Path::new(&path);
    let settings = get_settings_from_path(path)?;
    let factory = DefaultWorkspaceLoaderFactory;
    let loader = factory
        .create(&path)
        .map_err(|_e| Error::Atom("Failed to create workspace loader".into()))?;
    let workspace = loader
        .load(
            &settings,
            &StoreFactories::default(),
            &default_working_copy_factories(),
        )
        .map_err(|_e| Error::Atom("Failed to load workspace".into()))?;

    Ok(ResourceArc::new(WorkspaceResource(Mutex::new(workspace))))
}

#[rustler::nif]
fn get_configs(path: String) -> Result<Vec<(String, String)>, Error> {
    let settings = get_settings_from_path(Path::new(&path))?;
    let mut annotated_values =
        resolved_config_values(&settings.config(), &ConfigNamePathBuf::root());
    annotated_values.retain(|annotated| !annotated.is_overridden);
    annotated_values.retain(|annotated| annotated.source != ConfigSource::Default);

    let annotated_values = annotated_values
        .iter()
        .map(|annotated| {
            (
                annotated.name.to_string(),
                annotated.value.clone().decorated("", "").to_string(),
            )
        })
        .collect();

    Ok(annotated_values)
}

#[rustler::nif]
fn simple_log(resource: ResourceArc<WorkspaceResource>) -> Result<Vec<Commit>, String> {
    let workspace = resource.0.try_lock().unwrap();
    let repo = workspace
        .repo_loader()
        .load_at_head()
        .map_err(|e| format!("Failed to load repository: {}", e))?;
    let working_copy_head = repo
        .view()
        .get_wc_commit_id(workspace.workspace_name())
        .ok_or_else(|| "No working copy head found".to_string())?;
    let mut current_commit = repo
        .store()
        .get_commit(working_copy_head)
        .map_err(|e| format!("Failed to get head commit: {}", e))?;

    let mut commits = Vec::new();

    for _ in 0..10 {
        let parents = current_commit.parent_ids();
        commits.push(commit_to_erl_commit(&repo, &current_commit));

        if let Some(parent_id) = parents.first() {
            current_commit = repo
                .store()
                .get_commit(parent_id)
                .map_err(|e| format!("Failed to get parent commit: {}", e))?;
        } else {
            break;
        }
    }

    Ok(commits)
}

rustler::init!("Elixir.Jj.Native");
