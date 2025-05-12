use self::members::{TeamAddArgs, TeamListArgs, TeamRemoveArgs};
use crate::command::teams::create::CreateTeamArgs;
use crate::command::teams::delete::DeleteTeamArgs;
use crate::command::teams::update::UpdateTeamArgs;
use anyhow::Result;
use clap::{Args, Subcommand};

mod create;
mod delete;
mod members;
mod update;

#[derive(Debug, Args)]
#[command(next_help_heading = "Team options")]
pub struct Teams {
    #[arg(help = "The name of the team.")]
    pub name: String,

    #[command(subcommand)]
    teams_commands: TeamsCommands,
}

#[derive(Subcommand, Debug)]
pub enum TeamsCommands {
    #[command(about = "Create a new team.")]
    Create(CreateTeamArgs),

    #[command(about = "Update an existing team.")]
    Update(UpdateTeamArgs),

    #[command(about = "Delete a team.")]
    Delete(DeleteTeamArgs),
    #[command(about = "List team members.", aliases = ["ls"])]
    List(TeamListArgs),

    #[command(about = "Add a new team member.")]
    Add(TeamAddArgs),

    #[command(about = "Remove a team member.")]
    Remove(TeamRemoveArgs),
}

impl Teams {
    pub async fn run(&self) -> Result<()> {
        match &self.teams_commands {
            TeamsCommands::List(args) => args.run(self.name.clone()).await,
            TeamsCommands::Add(args) => args.run(self.name.clone()).await,
            TeamsCommands::Remove(args) => args.run(self.name.clone()).await,
            TeamsCommands::Create(args) => args.run(self.name.clone()).await,
            TeamsCommands::Update(args) => args.run(self.name.clone()).await,
            TeamsCommands::Delete(args) => args.run(self.name.clone()).await,
        }
    }
}
