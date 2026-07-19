//! `/config` -- open the API provider setup form (WeepCode renamed `/login`).

use crate::app::actions::Action;
use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand};

pub struct ConfigCommand;

impl SlashCommand for ConfigCommand {
    fn name(&self) -> &str {
        "config"
    }

    fn aliases(&self) -> &[&str] {
        &["login"]
    }

    fn description(&self) -> &str {
        "Configure an API provider"
    }

    fn usage(&self) -> &str {
        "/config"
    }

    fn run(&self, _ctx: &mut CommandExecCtx, _args: &str) -> CommandResult {
        CommandResult::Action(Action::Login)
    }
}
