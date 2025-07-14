mod commit;
mod configs;
mod revset;
mod settings;
mod workspace;

use jj_lib::repo::Repo;
use jj_lib::revset::RevsetIteratorExt;
use rustler::{Error, ResourceArc};
use std::path::Path;

use crate::{
    commit::{commit_to_erl_commit, Commit},
    configs::{configs_to_tuple_list, resolve_configs},
    revset::get_revset,
    settings::get_settings_from_path,
    workspace::{
        get_workspace_data, get_workspace_settings, wrap_workspace_settings, WorkspaceArc,
        WorkspaceResource,
    },
};

#[rustler::nif]
fn get_workspace(path: String) -> Result<WorkspaceArc, Error> {
    get_workspace_settings(Path::new(&path)).map(wrap_workspace_settings)
}

#[rustler::nif]
fn get_configs(path: String) -> Result<Vec<(String, String)>, Error> {
    get_settings_from_path(Path::new(&path))
        .map(|settings| resolve_configs(&settings))
        .map(configs_to_tuple_list)
}

#[rustler::nif]
fn log(
    resource: ResourceArc<WorkspaceResource>,
    revset_str: String,
) -> Result<Vec<Commit>, String> {
    let workspace_data = get_workspace_data(&resource);
    let revset = get_revset(&workspace_data, &revset_str)?;

    // Referenced from cmd_log in jj/cli/src/commands/log/log.rs
    revset
        .iter()
        .commits(workspace_data.workspace_settings.repo_readonly.store())
        .map(|commit_or_err| {
            commit_or_err
                .map_err(|e| format!("Revset evaluation error: {e}"))
                .map(|commit| {
                    commit_to_erl_commit(
                        workspace_data.workspace_settings.repo_readonly.as_ref(),
                        &commit,
                    )
                })
        })
        .collect()
}

rustler::init!("Elixir.Jj.Native");
