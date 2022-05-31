use std::{
    collections::HashMap,
    fs::File,
};

use color_eyre::Result;
use repl_rs::{Command, Convert, Parameter, Repl, Value};

mod player;
mod track;
pub use player::Player;

fn play(args: HashMap<String, Value>, player: &mut Player) -> Result<Option<String>> {
    let path: String = args["path"].convert()?;
    let source = File::open(&path)?;
    player.try_play(source)?;
    Ok(Some(format!("Playing {}", &path)))
}

fn resume(_args: HashMap<String, Value>, player: &mut Player) -> Result<Option<String>> {
    player.resume();
    Ok(Some("Resumed".to_string()))
}

fn pause(_args: HashMap<String, Value>, player: &mut Player) -> Result<Option<String>> {
    player.pause();
    Ok(Some("Paused".to_string()))
}

fn stop(_args: HashMap<String, Value>, player: &mut Player) -> Result<Option<String>> {
    player.stop();
    Ok(Some("Stopped".to_string()))
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let player = Player::try_new()?;

    let mut repl = Repl::new(player)
        .with_name("Impact")
        .with_description("Small music player")
        .use_completion(true)
        .add_command(
            Command::new("play", play)
                .with_parameter(Parameter::new("path").set_required(true)?)?
                .with_help("Play the specific file"),
        )
        .add_command(Command::new("resume", resume).with_help("Resume the current track"))
        .add_command(Command::new("pause", pause).with_help("Pause the current track"))
        .add_command(Command::new("stop", stop).with_help("Stop the current track"));
    repl.run()?;
    Ok(())
}
