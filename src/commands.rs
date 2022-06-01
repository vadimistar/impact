use crate::{track::TrackData, Context};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use id3::{Tag, TagLike};
use path_absolutize::Absolutize;
use repl_rs::{Convert, Value};
use std::{collections::HashMap, fs::File, path::Path};

pub fn play(_args: HashMap<String, Value>, ctx: &mut Context) -> Result<Option<String>> {
    let id = ctx.selected_id.ok_or_else(|| eyre!("No selected tracks"))?;
    let track = ctx.database.track_by_id(id)?;

    println!("Playing: {}", track);
    ctx.player.try_play(File::open(&track.path)?)?;

    Ok(None)
}

pub fn resume(_args: HashMap<String, Value>, ctx: &mut Context) -> Result<Option<String>> {
    ctx.player.resume();
    Ok(Some("Resumed".to_string()))
}

pub fn pause(_args: HashMap<String, Value>, ctx: &mut Context) -> Result<Option<String>> {
    ctx.player.pause();
    Ok(Some("Paused".to_string()))
}

pub fn stop(_args: HashMap<String, Value>, ctx: &mut Context) -> Result<Option<String>> {
    ctx.player.stop();
    Ok(Some("Stopped".to_string()))
}

pub fn select(args: HashMap<String, Value>, ctx: &mut Context) -> Result<Option<String>> {
    let input: String = args["track"].convert()?;
    let track = ctx.database.track_by_unknown_format(&input)?;

    ctx.selected_id = Some(track.id);
    println!("Selected {}", track);

    Ok(None)
}

pub fn deselect(_args: HashMap<String, Value>, ctx: &mut Context) -> Result<Option<String>> {
    ctx.selected_id = None;
    println!("Deselected");

    Ok(None)
}

pub fn add(args: HashMap<String, Value>, ctx: &mut Context) -> Result<Option<String>> {
    let path: String = args["path"].convert()?;

    let path = Path::new(&path);
    let path = path.absolutize()?;
    let path = path.to_str().unwrap();

    // We don't store the track, if it is already stored
    if ctx
        .database
        .tracks()?
        .iter()
        .any(|track| track.path.eq(&path))
    {
        return Ok(Some("This track is already stored".to_string()));
    }

    let tag = Tag::read_from_path(path)?;
    let track = TrackData {
        id: 0,
        path: path.to_string(),
        title: tag.title().unwrap_or("").to_string(),
        artist: tag.artist().unwrap_or("").to_string(),
        album: tag.album().unwrap_or("").to_string(),
    };

    ctx.database.add(track)?;
    Ok(Some(format!("Added {} into the player", path)))
}

pub fn remove(_args: HashMap<String, Value>, ctx: &mut Context) -> Result<Option<String>> {
    let id = ctx.selected_id.ok_or_else(|| eyre!("No selected tracks"))?;
    ctx.database.remove(id)?;

    Ok(Some(format!("Removed: ID {}", id)))
}

pub fn list(_args: HashMap<String, Value>, ctx: &mut Context) -> Result<Option<String>> {
    println!("{:15} {:15} {:15} {:15}", "id", "title", "artist", "album");
    ctx.database.tracks()?.iter().for_each(|track| {
        println!(
            "{:15} {:15} {:15} {:15}",
            track.id, track.title, track.artist, track.album,
        )
    });
    Ok(None)
}
