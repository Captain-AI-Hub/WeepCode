//! `/usage` -- show credit usage.

use crate::app::actions::Action;
use crate::slash::command::{AppCtx, ArgItem, CommandExecCtx, CommandResult, SlashCommand};

/// Show coding credit usage.
///
/// `/usage`        -- show current credit usage
/// `/usage show`   -- same as above
///
/// WeepCode: there is no billing management page to link to; the command is
/// hidden entirely for API-key providers (see `set_usage_visible`).
pub struct UsageCommand;

impl SlashCommand for UsageCommand {
    fn name(&self) -> &str {
        "usage"
    }

    /// `/cost` is the minimal-mode name for the same credit-usage summary:
    /// it commits a usage/cost system block rather than opening a
    /// pane, so it's an alias rather than a separate command.
    fn aliases(&self) -> &[&str] {
        &["cost"]
    }

    fn description(&self) -> &str {
        "View credit usage or manage billing"
    }

    fn usage(&self) -> &str {
        "/usage [show|manage]"
    }

    fn takes_args(&self) -> bool {
        true
    }

    fn arg_placeholder(&self) -> Option<&str> {
        Some("show | manage")
    }

    fn suggest_args(&self, _ctx: &AppCtx, _args_query: &str) -> Option<Vec<ArgItem>> {
        Some(vec![ArgItem {
            display: "show".to_string(),
            match_text: "show".to_string(),
            insert_text: "show".to_string(),
            description: "View credit usage".to_string(),
        }])
    }

    fn run(&self, _ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        let arg = args.trim();
        match arg {
            "" | "show" => CommandResult::Action(Action::ShowUsage),
            _ => CommandResult::Error(format!(
                "Unknown argument: {arg}. Use /usage show"
            )),
        }
    }
}
