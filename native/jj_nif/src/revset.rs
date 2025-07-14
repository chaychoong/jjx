use jj_cli::revset_util::default_symbol_resolver;
use jj_lib::revset::{optimize, parse, Revset, RevsetDiagnostics};

use crate::workspace::WorkspaceData;

pub fn get_revset<'a>(
    workspace_data: &'a WorkspaceData,
    revset_str: &str,
) -> Result<Box<dyn Revset + 'a>, String> {
    let revset_parse_context = workspace_data.get_revset_parse_context();
    let symbol_resolver = default_symbol_resolver(
        workspace_data.workspace_settings.repo_readonly.as_ref(),
        &revset_parse_context.extensions.symbol_resolvers(),
        &workspace_data.id_prefix_context,
    );

    // Referenced from cmd_debug_revset in jj/cli/src/commands/debug/revset.rs
    let mut diagnostics = RevsetDiagnostics::new();
    let expression_parsed = parse(&mut diagnostics, &revset_str, &revset_parse_context)
        .map_err(|e| format!("Revset parse error: {e}"))?;

    let mut expression_resolved = expression_parsed
        .resolve_user_expression(
            workspace_data.workspace_settings.repo_readonly.as_ref(),
            &symbol_resolver,
        )
        .map_err(|e| format!("Revset resolve error: {e}"))?;
    // Assume we want to optimize.
    expression_resolved = optimize(expression_resolved);

    let revset = expression_resolved
        .evaluate_unoptimized(workspace_data.workspace_settings.repo_readonly.as_ref())
        .map_err(|e| format!("Revset evaluate error: {e}"))?;

    Ok(revset)
}
