use std::sync::Arc;

use tonic::{Request, Response, Status};
use ytmusic_service_proto::ytmusic::v2::{self as pb, yt_music_server::YtMusic};

const UNIMPLEMENTED_MESSAGE: &str = "ytmusic.v2.YtMusic RPCs are not implemented yet";

pub struct MusicService {
    pub state: Arc<crate::state::AppState>,
}

#[tonic::async_trait]
impl YtMusic for MusicService {
    async fn search(
        &self,
        _request: Request<pb::SearchRequest>,
    ) -> Result<Response<pb::SearchResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn search_continuation(
        &self,
        _request: Request<pb::SearchContinuationRequest>,
    ) -> Result<Response<pb::SearchResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_watch_playlist(
        &self,
        _request: Request<pb::GetWatchPlaylistRequest>,
    ) -> Result<Response<pb::WatchPlaylistResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_watch_playlist_continuation(
        &self,
        _request: Request<pb::GetWatchPlaylistContinuationRequest>,
    ) -> Result<Response<pb::WatchPlaylistResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_song(
        &self,
        _request: Request<pb::GetSongRequest>,
    ) -> Result<Response<pb::GetSongResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_library_playlists(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::LibraryPlaylistsResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_library_playlists_continuation(
        &self,
        _request: Request<pb::ContinuationRequest>,
    ) -> Result<Response<pb::LibraryPlaylistsResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_account_info(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::AccountInfoResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_library_artists(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::LibraryArtistsResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_library_artists_continuation(
        &self,
        _request: Request<pb::ContinuationRequest>,
    ) -> Result<Response<pb::LibraryArtistsResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_library_albums(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::LibraryAlbumsResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_library_albums_continuation(
        &self,
        _request: Request<pb::ContinuationRequest>,
    ) -> Result<Response<pb::LibraryAlbumsResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_library_subscriptions(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::LibrarySubscriptionsResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_library_subscriptions_continuation(
        &self,
        _request: Request<pb::ContinuationRequest>,
    ) -> Result<Response<pb::LibrarySubscriptionsResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_library_channels(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::LibraryChannelsResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_library_channels_continuation(
        &self,
        _request: Request<pb::ContinuationRequest>,
    ) -> Result<Response<pb::LibraryChannelsResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_library_podcasts(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::LibraryPodcastsResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_library_podcasts_continuation(
        &self,
        _request: Request<pb::ContinuationRequest>,
    ) -> Result<Response<pb::LibraryPodcastsResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_library_songs(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::LibrarySongsResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_library_songs_continuation(
        &self,
        _request: Request<pb::ContinuationRequest>,
    ) -> Result<Response<pb::LibrarySongsResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_liked_songs(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::LikedSongsResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_liked_songs_continuation(
        &self,
        _request: Request<pb::ContinuationRequest>,
    ) -> Result<Response<pb::LikedSongsResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_saved_episodes(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::SavedEpisodesResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn get_saved_episodes_continuation(
        &self,
        _request: Request<pb::ContinuationRequest>,
    ) -> Result<Response<pb::SavedEpisodesResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }
}
