use crate::{auth_context::AuthContext, error::ServiceError};

pub struct YtMusicAdapter;

impl YtMusicAdapter {
    pub async fn search(
        auth: &AuthContext,
        query: ytmusicapi::SearchQuery,
    ) -> Result<
        ytmusicapi::Page<ytmusicapi::SearchResult, ytmusicapi::SearchContinuationToken>,
        ServiceError,
    > {
        auth.client
            .search(query)
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn search_continuation(
        auth: &AuthContext,
        token: ytmusicapi::SearchContinuationToken,
    ) -> Result<
        ytmusicapi::Page<ytmusicapi::SearchResult, ytmusicapi::SearchContinuationToken>,
        ServiceError,
    > {
        auth.client
            .search_continuation(token)
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_watch_playlist(
        auth: &AuthContext,
        query: ytmusicapi::WatchPlaylistQuery,
    ) -> Result<
        ytmusicapi::Page<ytmusicapi::WatchTrack, ytmusicapi::WatchPlaylistContinuationToken>,
        ServiceError,
    > {
        auth.client
            .get_watch_playlist(query)
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_watch_playlist_continuation(
        auth: &AuthContext,
        token: ytmusicapi::WatchPlaylistContinuationToken,
    ) -> Result<
        ytmusicapi::Page<ytmusicapi::WatchTrack, ytmusicapi::WatchPlaylistContinuationToken>,
        ServiceError,
    > {
        auth.client
            .get_watch_playlist_continuation(token)
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_song(
        auth: &AuthContext,
        video_id: String,
        signature_timestamp: u32,
    ) -> Result<ytmusicapi::GetSongResponse, ServiceError> {
        auth.client
            .get_song(video_id, signature_timestamp)
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_account_info(
        auth: &AuthContext,
    ) -> Result<ytmusicapi::AccountInfo, ServiceError> {
        auth.client
            .get_account_info()
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_library_playlists(
        auth: &AuthContext,
    ) -> Result<
        ytmusicapi::Page<
            ytmusicapi::LibraryPlaylist,
            ytmusicapi::LibraryPlaylistsContinuationToken,
        >,
        ServiceError,
    > {
        auth.client
            .get_library_playlists()
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_library_playlists_continuation(
        auth: &AuthContext,
        token: ytmusicapi::LibraryPlaylistsContinuationToken,
    ) -> Result<
        ytmusicapi::Page<
            ytmusicapi::LibraryPlaylist,
            ytmusicapi::LibraryPlaylistsContinuationToken,
        >,
        ServiceError,
    > {
        auth.client
            .get_library_playlists_continuation(token)
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_library_artists(
        auth: &AuthContext,
    ) -> Result<
        ytmusicapi::Page<ytmusicapi::LibraryArtist, ytmusicapi::LibraryArtistsContinuationToken>,
        ServiceError,
    > {
        auth.client
            .get_library_artists()
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_library_artists_continuation(
        auth: &AuthContext,
        token: ytmusicapi::LibraryArtistsContinuationToken,
    ) -> Result<
        ytmusicapi::Page<ytmusicapi::LibraryArtist, ytmusicapi::LibraryArtistsContinuationToken>,
        ServiceError,
    > {
        auth.client
            .get_library_artists_continuation(token)
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_library_albums(
        auth: &AuthContext,
    ) -> Result<
        ytmusicapi::Page<ytmusicapi::LibraryAlbum, ytmusicapi::LibraryAlbumsContinuationToken>,
        ServiceError,
    > {
        auth.client
            .get_library_albums()
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_library_albums_continuation(
        auth: &AuthContext,
        token: ytmusicapi::LibraryAlbumsContinuationToken,
    ) -> Result<
        ytmusicapi::Page<ytmusicapi::LibraryAlbum, ytmusicapi::LibraryAlbumsContinuationToken>,
        ServiceError,
    > {
        auth.client
            .get_library_albums_continuation(token)
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_library_subscriptions(
        auth: &AuthContext,
    ) -> Result<
        ytmusicapi::Page<
            ytmusicapi::LibrarySubscription,
            ytmusicapi::LibrarySubscriptionsContinuationToken,
        >,
        ServiceError,
    > {
        auth.client
            .get_library_subscriptions()
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_library_subscriptions_continuation(
        auth: &AuthContext,
        token: ytmusicapi::LibrarySubscriptionsContinuationToken,
    ) -> Result<
        ytmusicapi::Page<
            ytmusicapi::LibrarySubscription,
            ytmusicapi::LibrarySubscriptionsContinuationToken,
        >,
        ServiceError,
    > {
        auth.client
            .get_library_subscriptions_continuation(token)
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_library_channels(
        auth: &AuthContext,
    ) -> Result<
        ytmusicapi::Page<ytmusicapi::LibraryChannel, ytmusicapi::LibraryChannelsContinuationToken>,
        ServiceError,
    > {
        auth.client
            .get_library_channels()
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_library_channels_continuation(
        auth: &AuthContext,
        token: ytmusicapi::LibraryChannelsContinuationToken,
    ) -> Result<
        ytmusicapi::Page<ytmusicapi::LibraryChannel, ytmusicapi::LibraryChannelsContinuationToken>,
        ServiceError,
    > {
        auth.client
            .get_library_channels_continuation(token)
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_library_podcasts(
        auth: &AuthContext,
    ) -> Result<
        ytmusicapi::Page<ytmusicapi::LibraryPodcast, ytmusicapi::LibraryPodcastsContinuationToken>,
        ServiceError,
    > {
        auth.client
            .get_library_podcasts()
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_library_podcasts_continuation(
        auth: &AuthContext,
        token: ytmusicapi::LibraryPodcastsContinuationToken,
    ) -> Result<
        ytmusicapi::Page<ytmusicapi::LibraryPodcast, ytmusicapi::LibraryPodcastsContinuationToken>,
        ServiceError,
    > {
        auth.client
            .get_library_podcasts_continuation(token)
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_library_songs(
        auth: &AuthContext,
    ) -> Result<
        ytmusicapi::Page<ytmusicapi::LibrarySong, ytmusicapi::LibrarySongsContinuationToken>,
        ServiceError,
    > {
        auth.client
            .get_library_songs()
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_library_songs_continuation(
        auth: &AuthContext,
        token: ytmusicapi::LibrarySongsContinuationToken,
    ) -> Result<
        ytmusicapi::Page<ytmusicapi::LibrarySong, ytmusicapi::LibrarySongsContinuationToken>,
        ServiceError,
    > {
        auth.client
            .get_library_songs_continuation(token)
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_liked_songs(
        auth: &AuthContext,
    ) -> Result<ytmusicapi::LikedSongsPage, ServiceError> {
        auth.client
            .get_liked_songs()
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_liked_songs_continuation(
        auth: &AuthContext,
        token: ytmusicapi::LikedSongsContinuationToken,
    ) -> Result<ytmusicapi::LikedSongsPage, ServiceError> {
        auth.client
            .get_liked_songs_continuation(token)
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_saved_episodes(
        auth: &AuthContext,
    ) -> Result<ytmusicapi::SavedEpisodesPage, ServiceError> {
        auth.client
            .get_saved_episodes()
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_saved_episodes_continuation(
        auth: &AuthContext,
        token: ytmusicapi::SavedEpisodesContinuationToken,
    ) -> Result<ytmusicapi::SavedEpisodesPage, ServiceError> {
        auth.client
            .get_saved_episodes_continuation(token)
            .await
            .map_err(ServiceError::YtMusic)
    }
}
