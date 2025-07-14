use jj_cli::revset_util::load_revset_aliases;
use jj_cli::ui::Ui;
use jj_lib::id_prefix::IdPrefixContext;
use jj_lib::repo::{ReadonlyRepo, StoreFactories};
use jj_lib::repo_path::RepoPathUiConverter;
use jj_lib::revset::{
    RevsetAliasesMap, RevsetExtensions, RevsetParseContext, RevsetWorkspaceContext,
};
use jj_lib::settings::UserSettings;
use jj_lib::workspace::{
    default_working_copy_factories, DefaultWorkspaceLoaderFactory, Workspace,
    WorkspaceLoaderFactory,
};
use rustler::{Error, Resource, ResourceArc};
use std::collections::HashMap;
use std::default::Default;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::settings::get_settings_from_path;

pub struct WorkspaceSettings {
    pub workspace: Workspace,
    pub settings: UserSettings,
    pub repo_readonly: Arc<ReadonlyRepo>,
    path_converter: RepoPathUiConverter,
    revset_aliases_map: RevsetAliasesMap,
}

pub struct WorkspaceResource(pub Mutex<WorkspaceSettings>);

#[rustler::resource_impl]
impl Resource for WorkspaceResource {}

pub type WorkspaceArc = ResourceArc<WorkspaceResource>;
pub type UnwrappedWorkspaceSettings<'a> = MutexGuard<'a, WorkspaceSettings>;

pub struct WorkspaceData<'a> {
    pub workspace_settings: UnwrappedWorkspaceSettings<'a>,
    pub revset_extensions: Arc<RevsetExtensions>,
    pub id_prefix_context: IdPrefixContext,
}

pub fn get_workspace_settings(path: &Path) -> Result<WorkspaceSettings, Error> {
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

    let repo_readonly = workspace.repo_loader().load_at_head().unwrap();
    let path_converter = RepoPathUiConverter::Fs {
        cwd: workspace.workspace_root().to_owned(),
        base: workspace.workspace_root().to_owned(),
    };
    let revset_aliases_map = load_revset_aliases(&Ui::null(), settings.config())
        .map_err(|_e| Error::Atom("Failed to load revset aliases".into()))?;

    Ok(WorkspaceSettings {
        workspace,
        settings,
        repo_readonly,
        path_converter,
        revset_aliases_map,
    })
}

pub fn wrap_workspace_settings(workspace_settings: WorkspaceSettings) -> WorkspaceArc {
    ResourceArc::new(WorkspaceResource(Mutex::new(workspace_settings)))
}

pub fn get_workspace_data(resource: &WorkspaceArc) -> WorkspaceData {
    let workspace_settings = resource.0.lock().unwrap();
    let revset_extensions = Arc::new(RevsetExtensions::default());
    let id_prefix_context = IdPrefixContext::new(revset_extensions.clone());
    WorkspaceData {
        workspace_settings,
        revset_extensions,
        id_prefix_context,
    }
}

impl<'a> WorkspaceData<'a> {
    // Referenced from parse_context in gg/src-tauri/src/worker/gui_util.rs
    pub fn get_revset_parse_context(&'a self) -> RevsetParseContext<'a> {
        let workspace_context = RevsetWorkspaceContext {
            path_converter: &self.workspace_settings.path_converter,
            workspace_name: self.workspace_settings.workspace.workspace_name(),
        };

        // Create a RevsetParseContext
        RevsetParseContext {
            aliases_map: &self.workspace_settings.revset_aliases_map,
            local_variables: HashMap::new(),
            user_email: self.workspace_settings.settings.user_email(),
            date_pattern_context: chrono::Local::now().into(),
            extensions: &self.revset_extensions,
            workspace: Some(workspace_context),
        }
    }
}
