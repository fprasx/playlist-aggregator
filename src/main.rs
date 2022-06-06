use std::collections::HashSet;
use futures_util::StreamExt;
use rspotify::model::PlayableItem::{Episode, Track};
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Credentials, OAuth};

#[tokio::main]
async fn main() {
    let creds = Credentials::from_env().unwrap();

    let oauth = OAuth::from_env(scopes!(
        "playlist-read-collaborative",
        "playlist-read-private",
        "user-library-read"
    ))
    .unwrap();

    let mut spotify = AuthCodeSpotify::new(creds, oauth);

    // Obtaining the access token
    let url = spotify.get_authorize_url(false).unwrap();
    spotify.prompt_for_token(&url).await.unwrap();

    let mut tracks = HashSet::<String>::new();

    add_tracks_from_playlists(&spotify, &mut tracks).await;
    add_tracks_from_liked_songs(&spotify, &mut tracks).await;

    println!("{:#?}", tracks);
}

async fn add_tracks_from_liked_songs(spotify: &AuthCodeSpotify, tracks: &mut HashSet<String>) {
    // Note: cannot include the funciton call in the while loop condition!!!
    // It will just return the first track every time, as it's making a new api call every loop iteration
    let mut saved_tracks = spotify.current_user_saved_tracks(None);
    while let Some(Ok(t)) = saved_tracks.next().await {
        tracks.insert(t.track.name);
    }
}

async fn add_tracks_from_playlists(spotify: &AuthCodeSpotify, tracks: &mut HashSet<String>) {
    // Note: cannot include the funciton call in the while loop condition!!!
    // It will just return the first playlist every time, as it's making a new api call every loop iteration
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
}
