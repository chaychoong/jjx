use jj_lib::repo::{ReadonlyRepo, Repo};
use rustler::NifStruct;

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

pub fn commit_to_erl_commit(repo: &ReadonlyRepo, commit: &jj_lib::commit::Commit) -> Commit {
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
