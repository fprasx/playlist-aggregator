use futures_util::StreamExt;
use rspotify::model::PlaylistId;
use rspotify::model::PlayableItem::{Episode, Track};
use rspotify::{prelude::*, scopes, AuthCodePkceSpotify, Config, Credentials, OAuth, ClientError};
use core::fmt;
use std::collections::HashSet;
use std::error::Error;

type GeneralResult<T> = Result<T, ProgramError>;

#[derive(Debug)]
enum ProgramError {
    RSpotifyError(ClientError),
    LogicError(String)
}

impl fmt::Display for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgramError::RSpotifyError(e) => write!(f, "rspotify error: {}", e),
            ProgramError::LogicError(e) => write!(f, "Error: {}", e),
        }
    }
}

impl Error for ProgramError {}

#[tokio::main]
async fn main() {
    let creds = Credentials::from_env().unwrap();

    let oauth = OAuth::from_env(scopes!(
        "playlist-read-collaborative",
        "playlist-read-private",
        "playlist-modify-public",
        "playlist-modify-private",
        "user-library-read"
    ))
    .unwrap();

    let config = Config {
        token_cached: true,
        token_refreshing: true,
        ..Default::default()
    };

    let mut spotify = AuthCodePkceSpotify::with_config(creds, oauth, config);
    let url = spotify.get_authorize_url(None).unwrap();

    spotify.prompt_for_token(&url).await.unwrap();

    let mut tracks = HashSet::<String>::new();

    // add_tracks_from_playlists(&spotify, &mut tracks).await;
    // add_tracks_from_liked_songs(&spotify, &mut tracks).await;
    match create_playlist(
        &spotify,
        "Test from Rust",
        Some(false),
        Some(false),
        Some("A test playlist from Rust"),
    )
    .await {
        Ok(id) => println!("Playlist <{}> successfully created", id),
        Err(e) => println!("{}", e)
    }

    println!("You have {} songs!", tracks.len());
}

async fn create_playlist(
    spotify: &impl OAuthClient,
    name: &str,
    collaborative: Option<bool>,
    public: Option<bool>,
    description: Option<&str>,
) -> GeneralResult<PlaylistId> {
    let mut playlists = spotify.current_user_playlists();
    while let Some(Ok(p)) = playlists.next().await {
        let playlist = spotify.playlist(&p.id, None, None).await.unwrap();
        if playlist.name == name {
            return Err(ProgramError::LogicError(format!("Playlist <{}> already exists!", name)));
        }
    }

    let id = &spotify.me().await.unwrap().id;
    match spotify
        .user_playlist_create(id, name, public, collaborative, description)
        .await
    {
        Ok(p) => Ok(p.id),
        Err(e) => Err(ProgramError::RSpotifyError(e))
    }
}

async fn get_tracks_from_liked_songs(spotify: &impl OAuthClient) -> HashSet<String> {
    // Note: cannot include the funciton call in the while loop condition!!!
    // It will just return the first track every time, as it's making a new api call every loop iteration
    let mut tracks = HashSet::new();
    let mut saved_tracks = spotify.current_user_saved_tracks(None);
    while let Some(Ok(t)) = saved_tracks.next().await {
        tracks.insert(t.track.name);
    }
    tracks
}

async fn get_tracks_from_playlists(spotify: &impl OAuthClient) -> HashSet<String>{
    // Note: cannot include the funciton call in the while loop condition!!!
    // It will just return the first playlist every time, as it's making a new api call every loop iteration
    let mut tracks = HashSet::new();
    let mut playlists = spotify.current_user_playlists();
    while let Some(Ok(p)) = playlists.next().await {
        let playlist = spotify.playlist(&p.id, None, None).await.unwrap();
        for item in playlist.tracks.items {
            match item.track.unwrap() {
                Track(t) => {
                    tracks.insert(t.name);
                }
                Episode(_) => (),
            }
        }
    }
    tracks
}
