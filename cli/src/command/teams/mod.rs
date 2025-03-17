use self::members::{TeamAddArgs, TeamListArgs, TeamRemoveArgs};
use crate::command::teams::create::CreateTeamArgs;
use anyhow::Result;
use clap::{Args, Subcommand};

mod create;
mod members;

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
        }
    }
}
