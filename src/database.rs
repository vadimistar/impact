use std::path::Path;

use color_eyre::{eyre::eyre, Result};
use path_absolutize::Absolutize;
use rusqlite::{params, Connection};

use crate::track::TrackData;

pub struct Database {
    conn: Connection,
    tracks: Option<Vec<TrackData>>,
}

impl Database {
    pub fn try_new(path: &str) -> Result<Database> {
        let conn = Connection::open(path)?;
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
        Ok(Database { conn, tracks: None })
    }

    pub fn update_tracks(&mut self) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, path, title, artist, album FROM tracks")?;
        let track_iter = stmt.query_map([], |row| {
            Ok(TrackData {
                id: row.get(0)?,
                path: row.get(1)?,
                title: row.get(2)?,
                artist: row.get(3)?,
                album: row.get(4)?,
            })
        })?;
        let tracks = track_iter.map(|track| track.unwrap()).collect();
        self.tracks = Some(tracks);
        Ok(())
    }

    pub fn tracks(&mut self) -> Result<&[TrackData]> {
        loop {
            if let Some(ref tracks) = self.tracks {
                return Ok(tracks);
            }
            self.update_tracks()?;
        }
    }

    fn track_by_condition(&mut self, f: impl FnMut(&&TrackData) -> bool) -> Result<&TrackData> {
        loop {
            if let Some(ref tracks) = self.tracks {
                return tracks
                    .iter()
                    .find(f)
                    .ok_or_else(|| eyre!("Unknown track"));
            }
            self.update_tracks()?;
        }
    }

    pub fn track_by_id(&mut self, id: i32) -> Result<&TrackData> {
        self.track_by_condition(|track| track.id.eq(&id))
    }

    pub fn track_by_path(&mut self, path: &str) -> Result<&TrackData> {
        self.track_by_condition(|track| track.path.eq(&path))
    }

    pub fn track_by_artist_and_title(&mut self, artist: &str, title: &str) -> Result<&TrackData> {
        self.track_by_condition(|track| track.artist.eq(&artist) && track.title.eq(&title))
    }

    pub fn track_by_title(&mut self, title: &str) -> Result<&TrackData> {
        self.track_by_condition(|track| track.title.eq(&title))
    }

    pub fn track_by_unknown_format(&mut self, input: &str) -> Result<&TrackData> {
        if let Ok(id) = input.parse::<i32>() {
            return self.track_by_id(id);
        }

        let path = Path::new(input);
        if path.exists() {
            let path = path.absolutize().unwrap();
            let path = path.to_str().unwrap();
            return self.track_by_path(path);
        }

        if input.contains('-') {
            if let Some((artist, title)) = input.split_once('-') {
                let artist = artist.trim();
                let title = title.trim();
                return self.track_by_artist_and_title(artist, title);
            }
        }

        let title = input.trim();
        self.track_by_title(title)
    }

    pub fn add(&mut self, track: TrackData) -> Result<()> {
        self.conn.execute(
            "INSERT INTO tracks (path, title, artist, album) VALUES (?1, ?2, ?3, ?4)",
            params![track.path, track.title, track.artist, track.album],
        )?;
        Ok(())
    }

    pub fn remove(&mut self, id: i32) -> Result<()> {
        self.conn
            .execute("DELETE FROM tracks WHERE id = ?1", params![id])?;
        Ok(())
    }
}
