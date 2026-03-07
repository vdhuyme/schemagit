mod args;
mod commands;
mod output;

use anyhow::Result;
use args::{Cli, Commands, DocsSubcommand, GraphSubcommand};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Snapshot {
            driver,
            connection,
            output,
        } => commands::snapshot::execute(driver, &connection, &output).await,

        Commands::Diff {
            old,
            new,
            snapshot_dir,
            format,
            output,
            yes,
            no_create_dir,
        } => commands::diff::execute(
            &old,
            &new,
            &snapshot_dir,
            &format,
            output.as_deref(),
            yes,
            no_create_dir,
        ),

        Commands::Migrate {
            old,
            new,
            snapshot_dir,
            output,
            yes,
            no_create_dir,
        } => commands::migrate::execute(
            &old,
            &new,
            &snapshot_dir,
            output.as_deref(),
            yes,
            no_create_dir,
        ),

        Commands::Status {
            driver,
            connection,
            snapshots,
            output,
            yes,
            no_create_dir,
        } => {
            commands::status::execute(
                driver,
                &connection,
                &snapshots,
                output.as_deref(),
                yes,
                no_create_dir,
            )
            .await
        }

        Commands::List {
            directory,
            output,
            yes,
            no_create_dir,
        } => commands::list::execute(
            &directory,
            output.as_deref(),
            yes,
            no_create_dir,
        ),

        Commands::Snapshots {
            directory,
            output,
            yes,
            no_create_dir,
        } => commands::snapshots::execute(
            &directory,
            output.as_deref(),
            yes,
            no_create_dir,
        ),

        Commands::History {
            directory,
            output,
            yes,
            no_create_dir,
        } => commands::history::execute(
            &directory,
            output.as_deref(),
            yes,
            no_create_dir,
        ),

        Commands::Show {
            snapshot,
            directory,
            output,
            yes,
            no_create_dir,
        } => commands::show::execute(
            &snapshot,
            &directory,
            output.as_deref(),
            yes,
            no_create_dir,
        ),

        Commands::Summary {
            snapshot,
            directory,
            output,
            yes,
            no_create_dir,
        } => commands::summary::execute(
            &snapshot,
            &directory,
            output.as_deref(),
            yes,
            no_create_dir,
        ),

        Commands::Graph { subcommand } => match subcommand {
            GraphSubcommand::Render {
                snapshot,
                directory,
                format,
                output,
                yes,
                no_create_dir,
            } => commands::graph::execute(
                &snapshot,
                &directory,
                &format,
                output.as_deref(),
                yes,
                no_create_dir,
            ),
            GraphSubcommand::Serve {
                snapshot,
                directory,
                port,
            } => commands::graph::serve(&snapshot, &directory, port).await,
        },

        Commands::Export {
            snapshot,
            directory,
            format,
            output,
            yes,
            no_create_dir,
        } => commands::export::execute(
            &snapshot,
            &directory,
            &format,
            output.as_deref(),
            yes,
            no_create_dir,
        ),

        Commands::Validate {
            snapshot,
            directory,
            output,
            yes,
            no_create_dir,
        } => commands::validate::execute(
            &snapshot,
            &directory,
            output.as_deref(),
            yes,
            no_create_dir,
        ),

        Commands::Tag {
            snapshot,
            tag,
            directory,
        } => commands::tag::execute(&snapshot, &tag, &directory),

        Commands::Docs { subcommand } => match subcommand {
            DocsSubcommand::Generate {
                snapshot,
                directory,
                output,
            } => commands::docs::generate(&snapshot, &directory, &output),
        },

        Commands::Timeline {
            directory,
            format,
            output,
        } => {
            commands::timeline::execute(&directory, &format, output.as_deref())
        }
    }
}
