use std::sync::Arc;

use tonic::{Request, Response, Status};
use ytmusic_service_proto::ytmusic::v2::{self as pb, yt_music_server::YtMusic};

pub struct MusicService {
    pub state: Arc<crate::state::AppState>,
}

#[tonic::async_trait]
impl YtMusic for MusicService {
    async fn search(
        &self,
        request: Request<pb::SearchRequest>,
    ) -> Result<Response<pb::SearchResponse>, Status> {
        let query = search_request_to_query(request.into_inner())?;
        let page = self.state.music.search(query).await.map_err(|source| {
            crate::error::map_service_error("ytmusic", &crate::error::ServiceError::YtMusic(source))
        })?;
        Ok(Response::new(
            crate::servers::music_mapping::search_page_to_proto(page),
        ))
    }

    async fn search_continuation(
        &self,
        request: Request<pb::SearchContinuationRequest>,
    ) -> Result<Response<pb::SearchResponse>, Status> {
        let token = continuation_token(
            request.into_inner().token,
            "search continuation token must not be empty",
            ytmusicapi::SearchContinuationToken::new,
        )?;
        let page = self
            .state
            .music
            .search_continuation(token)
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::search_page_to_proto(page),
        ))
    }

    async fn get_watch_playlist(
        &self,
        request: Request<pb::GetWatchPlaylistRequest>,
    ) -> Result<Response<pb::WatchPlaylistResponse>, Status> {
        let query = watch_playlist_request_to_query(request.into_inner())?;
        let page = self
            .state
            .music
            .get_watch_playlist(query)
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::watch_page_to_proto(page),
        ))
    }

    async fn get_watch_playlist_continuation(
        &self,
        request: Request<pb::GetWatchPlaylistContinuationRequest>,
    ) -> Result<Response<pb::WatchPlaylistResponse>, Status> {
        let token = continuation_token(
            request.into_inner().token,
            "watch playlist continuation token must not be empty",
            ytmusicapi::WatchPlaylistContinuationToken::new,
        )?;
        let page = self
            .state
            .music
            .get_watch_playlist_continuation(token)
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::watch_page_to_proto(page),
        ))
    }

    async fn get_song(
        &self,
        request: Request<pb::GetSongRequest>,
    ) -> Result<Response<pb::GetSongResponse>, Status> {
        let video_id =
            required_non_empty(request.into_inner().video_id, "video_id must not be empty")?;
        let signature_timestamp = self
            .state
            .cipher
            .signature_timestamp()
            .await
            .map_err(|source| crate::error::map_service_error("cipher", &source))?;
        let song = self
            .state
            .music
            .get_song(video_id, signature_timestamp)
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::song_response_to_proto(song),
        ))
    }

    async fn get_library_playlists(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::LibraryPlaylistsResponse>, Status> {
        let page = self
            .state
            .music
            .get_library_playlists()
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::library_playlists_page_to_proto(page),
        ))
    }

    async fn get_library_playlists_continuation(
        &self,
        request: Request<pb::ContinuationRequest>,
    ) -> Result<Response<pb::LibraryPlaylistsResponse>, Status> {
        let token = continuation_token(
            request.into_inner().token,
            "library playlists continuation token must not be empty",
            ytmusicapi::LibraryPlaylistsContinuationToken::new,
        )?;
        let page = self
            .state
            .music
            .get_library_playlists_continuation(token)
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::library_playlists_page_to_proto(page),
        ))
    }

    async fn get_account_info(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::AccountInfoResponse>, Status> {
        let account_info = self
            .state
            .music
            .get_account_info()
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::account_info_to_proto(account_info),
        ))
    }

    async fn get_library_artists(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::LibraryArtistsResponse>, Status> {
        let page = self
            .state
            .music
            .get_library_artists()
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::library_artists_page_to_proto(page),
        ))
    }

    async fn get_library_artists_continuation(
        &self,
        request: Request<pb::ContinuationRequest>,
    ) -> Result<Response<pb::LibraryArtistsResponse>, Status> {
        let token = continuation_token(
            request.into_inner().token,
            "library artists continuation token must not be empty",
            ytmusicapi::LibraryArtistsContinuationToken::new,
        )?;
        let page = self
            .state
            .music
            .get_library_artists_continuation(token)
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::library_artists_page_to_proto(page),
        ))
    }

    async fn get_library_albums(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::LibraryAlbumsResponse>, Status> {
        let page = self
            .state
            .music
            .get_library_albums()
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::library_albums_page_to_proto(page),
        ))
    }

    async fn get_library_albums_continuation(
        &self,
        request: Request<pb::ContinuationRequest>,
    ) -> Result<Response<pb::LibraryAlbumsResponse>, Status> {
        let token = continuation_token(
            request.into_inner().token,
            "library albums continuation token must not be empty",
            ytmusicapi::LibraryAlbumsContinuationToken::new,
        )?;
        let page = self
            .state
            .music
            .get_library_albums_continuation(token)
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::library_albums_page_to_proto(page),
        ))
    }

    async fn get_library_subscriptions(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::LibrarySubscriptionsResponse>, Status> {
        let page = self
            .state
            .music
            .get_library_subscriptions()
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::library_subscriptions_page_to_proto(page),
        ))
    }

    async fn get_library_subscriptions_continuation(
        &self,
        request: Request<pb::ContinuationRequest>,
    ) -> Result<Response<pb::LibrarySubscriptionsResponse>, Status> {
        let token = continuation_token(
            request.into_inner().token,
            "library subscriptions continuation token must not be empty",
            ytmusicapi::LibrarySubscriptionsContinuationToken::new,
        )?;
        let page = self
            .state
            .music
            .get_library_subscriptions_continuation(token)
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::library_subscriptions_page_to_proto(page),
        ))
    }

    async fn get_library_channels(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::LibraryChannelsResponse>, Status> {
        let page = self
            .state
            .music
            .get_library_channels()
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::library_channels_page_to_proto(page),
        ))
    }

    async fn get_library_channels_continuation(
        &self,
        request: Request<pb::ContinuationRequest>,
    ) -> Result<Response<pb::LibraryChannelsResponse>, Status> {
        let token = continuation_token(
            request.into_inner().token,
            "library channels continuation token must not be empty",
            ytmusicapi::LibraryChannelsContinuationToken::new,
        )?;
        let page = self
            .state
            .music
            .get_library_channels_continuation(token)
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::library_channels_page_to_proto(page),
        ))
    }

    async fn get_library_podcasts(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::LibraryPodcastsResponse>, Status> {
        let page = self
            .state
            .music
            .get_library_podcasts()
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::library_podcasts_page_to_proto(page),
        ))
    }

    async fn get_library_podcasts_continuation(
        &self,
        request: Request<pb::ContinuationRequest>,
    ) -> Result<Response<pb::LibraryPodcastsResponse>, Status> {
        let token = continuation_token(
            request.into_inner().token,
            "library podcasts continuation token must not be empty",
            ytmusicapi::LibraryPodcastsContinuationToken::new,
        )?;
        let page = self
            .state
            .music
            .get_library_podcasts_continuation(token)
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::library_podcasts_page_to_proto(page),
        ))
    }

    async fn get_library_songs(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::LibrarySongsResponse>, Status> {
        let page = self
            .state
            .music
            .get_library_songs()
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::library_songs_page_to_proto(page),
        ))
    }

    async fn get_library_songs_continuation(
        &self,
        request: Request<pb::ContinuationRequest>,
    ) -> Result<Response<pb::LibrarySongsResponse>, Status> {
        let token = continuation_token(
            request.into_inner().token,
            "library songs continuation token must not be empty",
            ytmusicapi::LibrarySongsContinuationToken::new,
        )?;
        let page = self
            .state
            .music
            .get_library_songs_continuation(token)
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::library_songs_page_to_proto(page),
        ))
    }

    async fn get_liked_songs(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::LikedSongsResponse>, Status> {
        let page = self.state.music.get_liked_songs().await.map_err(|source| {
            crate::error::map_service_error("ytmusic", &crate::error::ServiceError::YtMusic(source))
        })?;
        Ok(Response::new(
            crate::servers::music_mapping::liked_songs_page_to_proto(page),
        ))
    }

    async fn get_liked_songs_continuation(
        &self,
        request: Request<pb::ContinuationRequest>,
    ) -> Result<Response<pb::LikedSongsResponse>, Status> {
        let token = continuation_token(
            request.into_inner().token,
            "liked songs continuation token must not be empty",
            ytmusicapi::LikedSongsContinuationToken::new,
        )?;
        let page = self
            .state
            .music
            .get_liked_songs_continuation(token)
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::liked_songs_page_to_proto(page),
        ))
    }

    async fn get_saved_episodes(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::SavedEpisodesResponse>, Status> {
        let page = self
            .state
            .music
            .get_saved_episodes()
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::saved_episodes_page_to_proto(page),
        ))
    }

    async fn get_saved_episodes_continuation(
        &self,
        request: Request<pb::ContinuationRequest>,
    ) -> Result<Response<pb::SavedEpisodesResponse>, Status> {
        let token = continuation_token(
            request.into_inner().token,
            "saved episodes continuation token must not be empty",
            ytmusicapi::SavedEpisodesContinuationToken::new,
        )?;
        let page = self
            .state
            .music
            .get_saved_episodes_continuation(token)
            .await
            .map_err(|source| {
                crate::error::map_service_error(
                    "ytmusic",
                    &crate::error::ServiceError::YtMusic(source),
                )
            })?;
        Ok(Response::new(
            crate::servers::music_mapping::saved_episodes_page_to_proto(page),
        ))
    }
}

fn search_request_to_query(request: pb::SearchRequest) -> Result<ytmusicapi::SearchQuery, Status> {
    let query_text = required_non_empty(request.query, "query must not be empty")?;
    let mut query = ytmusicapi::SearchQuery::new(query_text);

    if let Some(filter) = request.filter {
        match pb::SearchFilter::try_from(filter) {
            Ok(pb::SearchFilter::Unspecified) => {}
            Ok(pb::SearchFilter::Songs) => {
                query = query.with_filter(ytmusicapi::SearchFilter::Songs);
            }
            Ok(pb::SearchFilter::Videos) => {
                query = query.with_filter(ytmusicapi::SearchFilter::Videos);
            }
            Ok(pb::SearchFilter::Albums) => {
                query = query.with_filter(ytmusicapi::SearchFilter::Albums);
            }
            Ok(pb::SearchFilter::Artists) => {
                query = query.with_filter(ytmusicapi::SearchFilter::Artists);
            }
            Ok(pb::SearchFilter::Playlists) => {
                query = query.with_filter(ytmusicapi::SearchFilter::Playlists);
            }
            Err(_) => {
                return Err(crate::error::map_invalid_argument(
                    "ytmusic",
                    format!("unknown search filter value: {filter}"),
                ));
            }
        }
    }

    if request.ignore_spelling {
        query = query.ignore_spelling();
    }

    Ok(query)
}

fn watch_playlist_request_to_query(
    request: pb::GetWatchPlaylistRequest,
) -> Result<ytmusicapi::WatchPlaylistQuery, Status> {
    let video_id = optional_non_empty(request.video_id, "video_id must not be empty")?;
    let playlist_id = optional_non_empty(request.playlist_id, "playlist_id must not be empty")?;

    if video_id.is_none() && playlist_id.is_none() {
        return Err(crate::error::map_invalid_argument(
            "ytmusic",
            "watch playlist query requires video_id or playlist_id",
        ));
    }

    if request.shuffle && playlist_id.is_none() {
        return Err(crate::error::map_invalid_argument(
            "ytmusic",
            "watch playlist shuffle requires playlist_id",
        ));
    }

    if request.radio && request.shuffle {
        return Err(crate::error::map_invalid_argument(
            "ytmusic",
            "watch playlist shuffle cannot be combined with radio",
        ));
    }

    let mut query = ytmusicapi::WatchPlaylistQuery::new();
    if let Some(video_id) = video_id {
        query = query.with_video_id(video_id);
    }
    if let Some(playlist_id) = playlist_id {
        query = query.with_playlist_id(playlist_id);
    }
    if request.radio {
        query = query.radio();
    }
    if request.shuffle {
        query = query.shuffle();
    }

    Ok(query)
}

fn required_non_empty(value: String, message: &'static str) -> Result<String, Status> {
    if value.trim().is_empty() {
        return Err(crate::error::map_invalid_argument("ytmusic", message));
    }

    Ok(value)
}

fn optional_non_empty(
    value: Option<String>,
    message: &'static str,
) -> Result<Option<String>, Status> {
    match value {
        Some(value) if value.trim().is_empty() => {
            Err(crate::error::map_invalid_argument("ytmusic", message))
        }
        other => Ok(other),
    }
}

fn continuation_token<T>(
    value: String,
    message: &'static str,
    build: impl FnOnce(String) -> T,
) -> Result<T, Status> {
    Ok(build(required_non_empty(value, message)?))
}
