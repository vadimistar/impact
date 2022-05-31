use std::{collections::HashMap, fs::File, path::Path};

use color_eyre::{eyre::eyre, Result};
use id3::{Tag, TagLike};
use repl_rs::{Command, Convert, Parameter, Repl, Value};

use path_absolutize::Absolutize;

mod player;
mod track;
pub use player::Player;
use rusqlite::{params, Connection};
use track::TrackData;

struct Context {
    player: Player,
    conn: Connection,
}

fn play_file(path: &str, player: &mut Player) -> Result<()> {
    let source = File::open(&path)?;
    player.try_play(source)?;
    Ok(())
}

fn track_datas(conn: &mut Connection) -> Result<Vec<TrackData>> {
    let mut stmt = conn.prepare("SELECT id, path, title, artist, album FROM tracks")?;
    let track_iter = stmt.query_map([], |row| {
        Ok(TrackData {
            id: row.get(0)?,
            path: row.get(1)?,
            title: row.get(2)?,
            artist: row.get(3)?,
            album: row.get(4)?,
        })
    })?;

    Ok(track_iter.map(|data| data.unwrap()).collect())
}

fn track_data(conn: &mut Connection, title: &str, artist: Option<&str>) -> Result<TrackData> {
    let track_datas = track_datas(conn)?;
    let matches: Vec<_> = track_datas
        .into_iter()
        .filter(
            |TrackData {
                 id: _,
                 path: _,
                 title: title_,
                 artist: _,
                 album: _,
             }| { title.eq(title_) },
        )
        .collect();

    match matches.len() {
        0 => Err(eyre!("Unknown track")),
        1 => Ok(matches.get(0).unwrap().clone()),
        _ => {
            if let Some(ref artist) = artist {
                return matches
                    .iter()
                    .find(
                        |TrackData {
                             id: _,
                             path: _,
                             title: _,
                             artist: artist_,
                             album: _,
                         }| artist.eq(artist_),
                    )
                    .map(|track| track.clone())
                    .ok_or_else(|| eyre!("Unknown track"));
            }
            Err(eyre!(
                "Multiple tracks with the same title exist, so artist has to be provided"
            ))
        }
    }
}

fn play(args: HashMap<String, Value>, ctx: &mut Context) -> Result<Option<String>> {
    let title: String = args["title"].convert()?;
    let artist: Option<String> = args.get("artist").map(|artist| artist.convert().unwrap());

    let track_data = track_data(&mut ctx.conn, &title, artist.as_deref())?;

    println!("Playing: {}", track_data);
    play_file(&track_data.path, &mut ctx.player)?;
    Ok(None)
}

fn resume(_args: HashMap<String, Value>, ctx: &mut Context) -> Result<Option<String>> {
    ctx.player.resume();
    Ok(Some("Resumed".to_string()))
}

fn pause(_args: HashMap<String, Value>, ctx: &mut Context) -> Result<Option<String>> {
    ctx.player.pause();
    Ok(Some("Paused".to_string()))
}

fn stop(_args: HashMap<String, Value>, ctx: &mut Context) -> Result<Option<String>> {
    ctx.player.stop();
    Ok(Some("Stopped".to_string()))
}

fn add(args: HashMap<String, Value>, ctx: &mut Context) -> Result<Option<String>> {
    let path: String = args["path"].convert()?;

    let path = Path::new(&path);
    let path = path.absolutize()?;
    let path = path.to_str().unwrap();

    fn track_data(path: &str) -> Result<TrackData> {
        let tag = Tag::read_from_path(path)?;

        Ok(TrackData {
            id: 0,
            path: path.to_string(),
            title: tag.title().unwrap_or_else(|| "").to_string(),
            artist: tag.artist().unwrap_or_else(|| "").to_string(),
            album: tag.album().unwrap_or_else(|| "").to_string(),
        })
    }

    // We don't store the track, if it is already stored
    if track_datas(&mut ctx.conn)?.iter().any(
        |TrackData {
             id: _,
             path: stored_path,
             title: _,
             artist: _,
             album: _,
         }| { stored_path == path },
    ) {
        return Ok(Some("This track is already stored".to_string()));
    }

    let track_data = track_data(path)?;

    ctx.conn.execute(
        "INSERT INTO tracks (path, title, artist, album) VALUES (?1, ?2, ?3, ?4)",
        params![
            track_data.path,
            track_data.title,
            track_data.artist,
            track_data.album
        ],
    )?;

    Ok(Some(format!("Added {} into the player", path)))
}

fn list(_args: HashMap<String, Value>, ctx: &mut Context) -> Result<Option<String>> {
    track_datas(&mut ctx.conn)?
        .into_iter()
        .for_each(|track_data| {
            println!(
                "{} ({:?}) {} - {} ({})",
                track_data.id,
                track_data.path,
                track_data.artist,
                track_data.title,
                track_data.album,
            );
        });
    Ok(None)
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let conn = Connection::open("./tracks.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tracks (
            id      INTEGER PRIMARY KEY,
            path    TEXT NOT NULL,
            title   TEXT NOT NULL,
            artist  TEXT NOT NULL,
            album   TEXT NOT NULL
        )",
        [],
    )?;

    let player = Player::try_new()?;
    let context = Context { player, conn };

    let mut repl = Repl::new(context)
        .with_name("Impact")
        .with_description("Small music player")
        .use_completion(true)
        .add_command(
            Command::new("play", play)
                .with_parameter(Parameter::new("title").set_required(true)?)?
                .with_parameter(Parameter::new("artist").set_required(false)?)?
                .with_help("Play the specific track in the player"),
        )
        .add_command(Command::new("resume", resume).with_help("Resume the current track"))
        .add_command(Command::new("pause", pause).with_help("Pause the current track"))
        .add_command(Command::new("stop", stop).with_help("Stop the current track"))
        .add_command(
            Command::new("add", add)
                .with_parameter(Parameter::new("path").set_required(true)?)?
                .with_help("Add the specific file into the player"),
        )
        .add_command(Command::new("list", list).with_help("List of all added tracks"));
    repl.run()?;
    Ok(())
}
