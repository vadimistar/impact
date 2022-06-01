use color_eyre::Result;
use database::Database;
use repl_rs::{Command, Parameter, Repl};

mod commands;
mod database;
mod player;
mod track;
pub use player::Player;

pub struct Context {
    player: Player,
    database: Database,
    selected_id: Option<i32>,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let player = Player::try_new()?;
    let context = Context {
        player,
        database: Database::try_new("./tracks.db")?,
        selected_id: None,
    };

    let mut repl = Repl::new(context)
        .with_name("Impact")
        .with_description("Small music player")
        .use_completion(true)
        .add_command(
            Command::new("play", commands::play)
                .with_help("Play the selected track in the player"),
        )
        .add_command(Command::new("resume", commands::resume).with_help("Resume the current track"))
        .add_command(Command::new("pause", commands::pause).with_help("Pause the current track"))
        .add_command(Command::new("stop", commands::stop).with_help("Stop the current track"))
        .add_command(
            Command::new("select", commands::select)
                .with_parameter(Parameter::new("track").set_required(true)?)?
                .with_help("Select the specific track from the player (id, title, title & author or file path have to be provided)"),
        )
        .add_command(Command::new("deselect", commands::deselect).with_help("Deselect the currently selected track"))
        .add_command(
            Command::new("add", commands::add)
                .with_parameter(Parameter::new("path").set_required(true)?)?
                .with_help("Add the specific file into the player"),
        )
        .add_command(
            Command::new("remove", commands::remove)
                .with_help("Remove the selected track from the player"),
        )
        .add_command(Command::new("list", commands::list).with_help("List of all added tracks"));
    repl.run()?;
    Ok(())
}
