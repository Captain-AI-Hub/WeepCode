//! `/add-model` -- open the API provider setup form (WeepCode).
//!
//! Renamed from `/config` so the upstream `/settings` alias `config` keeps
//! its original meaning; `/login` stays as a compatibility alias.

use crate::app::actions::Action;
use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand};

pub struct AddModelCommand;

impl SlashCommand for AddModelCommand {
    fn name(&self) -> &str {
        "add-model"
    }

    fn aliases(&self) -> &[&str] {
        &["login"]
    }

    fn description(&self) -> &str {
        "Add an API provider / model"
    }

    fn usage(&self) -> &str {
        "/add-model"
    }

    fn run(&self, _ctx: &mut CommandExecCtx, _args: &str) -> CommandResult {
        CommandResult::Action(Action::Login)
    }
}
