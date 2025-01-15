//! Reading through this file shows the entire process of running the application
use anyhow::{Context, Error, Result};
use color::*;
use futures_util::StreamExt;

use rspotify::model::{
    PlayableItem::{Episode, Track},
    PlaylistId, TrackId,
};
use rspotify::{prelude::*, scopes, AuthCodePkceSpotify, Config, Credentials, OAuth};

use std::fmt::{self, Display};
use std::io;
use std::time::Duration;
use std::{
    cmp,
    collections::{HashMap, HashSet},
    io::Write,
};

pub mod color;

// Can only add up to 100 tracks at a time (just a hard limit)
// Also sometimes this errors randomly, I think it's because a url is exceeding 2000 characters
// So we artifically cap at 50 tracks (lucky number)
const CHUNK_SIZE: usize = 50;
const LONG_DASH: &str = "─";
const CURSOR_LEFT: &str = "\x1B[1000D";
const CURSOR_HIDE: &str = "\x1B[?25l";
const CURSOR_SHOW: &str = "\x1B[?25h";

fn cprintln(s: impl fmt::Display, color: Color) {
    println!("{color}{s}{RESET}")
}

pub fn greet() {
    cprintln(
        r#"
      ─────────────────────────────────────┐ 
    ┌────────────────────────────────────┐ │ 
    │                                    │ │ 
    │  Welcome to playlist-aggregrator!  │ │ 
    │                                    │ │
    └────────────────────────────────────┘
    "#,
        CYAN,
    );
}

pub async fn authorize() -> Result<AuthCodePkceSpotify> {
    println!("{YELLOW}Authorizing . . . {RESET}");

    let creds = Credentials::from_env().unwrap();

    let oauth = OAuth::from_env(scopes!(
        "playlist-read-collaborative",
        "playlist-read-private",
        "playlist-modify-public",
        "playlist-modify-private" //
                                  //
    ))
    .unwrap();

    let config = Config {
        token_cached: true,
        token_refreshing: true,
        ..Default::default()
    };

    let mut spotify = AuthCodePkceSpotify::with_config(creds, oauth, config);

    // The URL the user enters into the terminal
    let url = spotify.get_authorize_url(None).unwrap();

    match spotify.prompt_for_token(&url).await {
        Ok(_) => (),
        // If the token is too old, refresh it
        Err(_) => spotify.refresh_token().await.unwrap(),
    }

    println!("{YELLOW}Authorization complete!{RESET}");

    Ok(spotify)
}

fn read_from_stdin(prompt: impl Display) -> String {
    println!("{YELLOW}{prompt}{RESET}");

    let mut name = String::new();

    io::stdin()
        .read_line(&mut name)
        .expect("{RED}failed to read from stdin{RESET}");

    name.trim().to_owned().to_lowercase()
}

async fn get_user_playlists(spotify: &impl OAuthClient) -> HashMap<String, PlaylistId> {
    let mut playlists = spotify.current_user_playlists();
    let mut names = HashMap::new();

    // loop through playlists, if one with same name exists, return an error
    while let Some(Ok(p)) = playlists.next().await {
        names.insert(p.name, p.id);
    }

    names
}

// Create a new playlist
//
// Returns an error if a playlist with the same name already exists
pub async fn create_playlist<'a>(
    spotify: &'a impl OAuthClient,
    description: Option<&str>,
) -> Result<(PlaylistId<'a>, String)> {
    let names = get_user_playlists(spotify).await;

    let mut name = read_from_stdin("What would you like your playlist to be called?");

    while names.contains_key(&name) {
        // The playlist already exists, do they want to overwirte
        let resp = read_from_stdin(format!(
            "You already have a playlist named <{GREEN}{}{YELLOW}>, would you like to overwrite that playlist? [y/n]",
            name
        ));

        // If overwrite is a go, return the ID of playlist with the name provided
        if resp == "y" || resp == "yes" {
            println!("{YELLOW}Proceeding with overwrite.{RESET}");
            let id = names.get(&name).unwrap();
            // Update the description
            spotify
                .playlist_change_detail(id.clone(), None, None, description, None)
                .await?;
            return Ok((id.clone(), name));
        } else {
            // Else, reprompt for a name
            name = read_from_stdin("What other name would you like to name your playlist?");
        }
    }

    // Should the new playlist be public
    let public = {
        let resp = read_from_stdin("Do you want to the playlist to be public? [y/n]");
        Some(resp == "y" || resp == "yes")
    };

    // Make the playlist
    let id = &spotify
        .me()
        .await
        .context("error getting user id, don't forget to add user in spotify developer dashboard")?
        .id;

    // Return the id of the newly created playlist
    let id = spotify
        .user_playlist_create(id.clone(), &name, public, Some(false), description)
        .await
        .map_err(Error::from)
        .map(|p| (p.id, name.clone()))
        .context("failed to create playlist");

    println!("{YELLOW}Created playlist <{GREEN}{name}{YELLOW}>{RESET}");

    id
}

// Helpers for summarizing playlists without passing around a bunch of tuple structs

struct PlaylistSummary {
    num_tracks: usize,
    runtime: Runtime,
}

struct Runtime {
    seconds: usize,
    minutes: usize,
    hours: usize,
}

impl From<Duration> for Runtime {
    fn from(dur: Duration) -> Self {
        let mut seconds = dur.as_secs() as usize;
        let hours = seconds / 3600;

        // Chop off all blocks of 3600 secs (1 hour)
        seconds %= 3600;
        let minutes = seconds / 60;

        // Chope off all blocks of 60 secs (1 minute)
        seconds %= 60;
        Self {
            seconds,
            minutes,
            hours,
        }
    }
}

impl Display for Runtime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{:03} {:02} {:02}",
            self.hours, self.minutes, self.seconds
        ))
    }
}
// Get all tracks from liked songs, return their combined length
async fn get_tracks_from_liked_songs(
    spotify: &impl OAuthClient,
    found: &mut HashSet<TrackId<'_>>,
) -> Runtime {
    let mut duration = Duration::from_secs(0);

    // Note: cannot include the function call in the while loop condition!!!
    // It will just return the first track every time, as it's making a new api call every loop iteration
    let mut saved_tracks = spotify.current_user_saved_tracks(None);

    while let Some(Ok(t)) = saved_tracks.next().await {
        found.insert(t.track.id.unwrap());
        duration = duration
            .checked_add(t.track.duration.to_std().unwrap())
            .unwrap();
    }
    duration.into()
}

// Return the number of tracks and duration of a playlist
async fn playlist_summary(spotify: &impl BaseClient, id: &PlaylistId<'_>) -> PlaylistSummary {
    let mut num_tracks = 0;
    let mut duration: Duration = Duration::from_secs(0);

    let mut p = spotify.playlist_items(id.clone(), None, None);

    // Paginate through the items
    while let Some(Ok(item)) = p.next().await {
        num_tracks += 1;

        // Failed at column 44
        let inner = match item.track {
            Some(track) => track,
            // TODO: maybe log this
            None => continue,
        };

        duration = duration
            .checked_add(match inner {
                Track(t) => t.duration.to_std().unwrap(),
                Episode(_) => Duration::from_secs(0),
            })
            .unwrap(); // You would need to have a very long playlist to overflow
    }

    PlaylistSummary {
        num_tracks,
        runtime: duration.into(),
    }
}

// Get all tracks from playlists
async fn get_tracks_from_playlists(
    spotify: &impl OAuthClient,
    found: &mut HashSet<TrackId<'_>>,
) -> Runtime {
    let mut total_duration: Duration = Duration::from_secs(0);

    println!("{:<79}Tracks    h  m  s", "");

    // Note: cannot include the function call in the while loop condition!!!
    // It will just return the first playlist every time, as it's making a new api call every loop iteration
    let mut playlists = spotify.current_user_playlists();
    while let Some(Ok(playlist)) = playlists.next().await {
        // TODO: check playlist id

        // For each playlist, print out a nice message
        let PlaylistSummary {
            num_tracks,
            runtime: duration,
        } = playlist_summary(spotify, &playlist.id).await;

        // TODO: Can't get padding right with brackets
        println!(
            "{CYAN}{:<80}{RESET} {GREEN}{:04}{RESET}  {YELLOW}{}{RESET}",
            playlist.name, num_tracks, duration
        );

        let mut items = spotify.playlist_items(playlist.id, None, None);

        // Paginate through the tracks
        while let Some(Ok(item)) = items.next().await {
            let t = match item.track {
                Some(it) => it,
                _ => continue,
            };
            if let Track(t) = t {
                if found.insert(match t.id {
                    Some(id) => id,
                    None => continue,
                }) {
                    // If the track wasn't found before, add its duration to the total
                    total_duration = total_duration
                        .checked_add(t.duration.to_std().unwrap())
                        .unwrap();
                }
            }
        }
    }

    total_duration.into()
}

pub async fn get_all_tracks(spotify: &AuthCodePkceSpotify) -> Vec<TrackId> {
    println!("{YELLOW}Retrieving tracks:{RESET}");

    let mut tracks = HashSet::<TrackId>::new();

    // This could be lower than the actual value if there are songs in liked songs that aren't on any playlist.
    // It would be too slow to make a request for each track in `tracks`, so we just assume most tracks are in a
    // playlist. This is probably pretty safe, especially if playlists have been aggregated before.
    let playlists_dur = get_tracks_from_playlists(spotify, &mut tracks).await;

    get_tracks_from_liked_songs(spotify, &mut tracks).await;

    // Print a summary of tracks
    cprintln(LONG_DASH.repeat(56), RED);
    println!(
        "{:<40} {GREEN}{:04} {YELLOW}~{}{RESET}",
        "Total",
        tracks.len(),
        playlists_dur
    );

    let tracks = tracks.into_iter().collect::<Vec<_>>();

    println!("{YELLOW}Tracks retrieved!{RESET}");

    tracks
}

pub async fn add_tracks_to_playlist(
    spotify: &impl OAuthClient,
    p: &PlaylistId<'_>,
    tracks: Vec<TrackId<'_>>,
    name: String,
) -> Result<()> {
    // Clear the playlist first in case we're overwriting
    spotify
        .playlist_replace_items(p.clone(), [])
        .await
        .context(format!("failed to clear playlist {name}"))?;

    // Defere error handling until all tracks have been (hopefully) added
    let mut errs = vec![];

    let mut added = 0;
    for chunk in tracks.chunks(CHUNK_SIZE) {
        match spotify
            .playlist_add_items(
                p.clone(),
                chunk.iter().map(|t| PlayableId::Track(t.clone())),
                None,
            )
            .await
        {
            Ok(_) => {
                added += CHUNK_SIZE;
                // Number of equal signs to put
                // Truncate it down if it exceeds 40 (the largest number of equal signs)
                let fill = cmp::min(40 * added / tracks.len(), 40);
                print!(
                    "{CURSOR_HIDE}{CURSOR_LEFT}{CYAN}[{RESET}{}>{}{CYAN}]{RESET}",
                    "=".repeat(fill.saturating_sub(1)),
                    " ".repeat(40 - fill)
                );
                // Not a big deal if we fail to flush
                let _ = io::stdout().flush();
            }
            Err(e) => {
                let error = Error::from(e).context(
                    "failed to add a chunk of tracks, you might want to delete the playlist",
                );
                errs.push(error);
            }
        }
    }

    // Handle all errors now
    if !errs.is_empty() {
        for err in errs {
            println!("{err}");
        }
        let delete = read_from_stdin(
            "Do you want to delete the playlist, since some tracks might be missing? [y/n]",
        );
        if delete == "y" || delete == "yes" {
            match spotify.playlist_unfollow(p.clone()).await {
                Ok(_) => println!("Deleted test from Rus<t>!"),
                Err(e) => {
                    // Restore the cursor
                    println!("{CURSOR_SHOW}");
                    return Err(anyhow::Error::from(e)
                        .context("failed to delete playlist as part of cleanup"));
                }
            }
        }
    }
    // Restore the cursor, finish printing the bar
    println!(
        "{CURSOR_LEFT}{CYAN}[{RESET}{}{CYAN}]{RESET}{CURSOR_SHOW}",
        "=".repeat(40)
    );
    println!("{YELLOW}Wrote all songs to <{GREEN}{name}{YELLOW}>!{RESET}");

    Ok(())
}
