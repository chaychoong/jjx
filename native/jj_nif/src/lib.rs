use jj_lib::{
    config::StackedConfig,
    object_id::ObjectId,
    repo::{Repo, StoreFactories},
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
    pub id: String,
    pub message_first_line: String,
    pub author_name: String,
    pub author_email: String,
}

pub struct WorkspaceResource(Mutex<Workspace>);

#[rustler::resource_impl]
impl Resource for WorkspaceResource {}

type WorkspaceArc = ResourceArc<WorkspaceResource>;

fn commit_to_erl_commit(commit: &jj_lib::commit::Commit) -> Commit {
    Commit {
        id: commit.id().hex(),
        message_first_line: commit
            .description()
            .lines()
            .next()
            .unwrap_or("")
            .to_string(),
        author_name: commit.author().name.clone(),
        author_email: commit.author().email.clone(),
    }
}

#[rustler::nif]
fn get_workspace(path: String) -> Result<WorkspaceArc, Error> {
    let path = Path::new(&path);
    let factory = DefaultWorkspaceLoaderFactory;
    let loader = factory
        .create(&path)
        .map_err(|_e| Error::Atom("Failed to create workspace loader".into()))?;
    let config = StackedConfig::with_defaults();
    let settings = UserSettings::from_config(config)
        .map_err(|_e| Error::Atom("Failed to create settings".into()))?;
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
        commits.push(commit_to_erl_commit(&current_commit));

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
