use ytmusic_service_proto::ytmusic::v2::{self as pb, search_result};

pub fn search_page_to_proto(
    page: ytmusicapi::Page<ytmusicapi::SearchResult, ytmusicapi::SearchContinuationToken>,
) -> pb::SearchResponse {
    pb::SearchResponse {
        items: page.items.into_iter().map(search_result_to_proto).collect(),
        continuation_token: page.continuation.map(|token| token.as_str().to_owned()),
    }
}

pub fn watch_page_to_proto(
    page: ytmusicapi::Page<ytmusicapi::WatchTrack, ytmusicapi::WatchPlaylistContinuationToken>,
) -> pb::WatchPlaylistResponse {
    pb::WatchPlaylistResponse {
        items: page.items.into_iter().map(watch_track_to_proto).collect(),
        continuation_token: page.continuation.map(|token| token.as_str().to_owned()),
    }
}

pub fn song_response_to_proto(song: ytmusicapi::GetSongResponse) -> pb::GetSongResponse {
    pb::GetSongResponse {
        video_details: Some(song_video_details_to_proto(song.video_details)),
        playability_status: Some(song_playability_status_to_proto(song.playability_status)),
        streaming_data: song.streaming_data.map(song_streaming_data_to_proto),
        microformat: song.microformat.map(song_microformat_to_proto),
    }
}

pub fn library_playlists_page_to_proto(
    page: ytmusicapi::Page<
        ytmusicapi::LibraryPlaylist,
        ytmusicapi::LibraryPlaylistsContinuationToken,
    >,
) -> pb::LibraryPlaylistsResponse {
    pb::LibraryPlaylistsResponse {
        items: page
            .items
            .into_iter()
            .map(library_playlist_item_to_proto)
            .collect(),
        continuation_token: page.continuation.map(|token| token.as_str().to_owned()),
    }
}

pub fn account_info_to_proto(account_info: ytmusicapi::AccountInfo) -> pb::AccountInfoResponse {
    pb::AccountInfoResponse {
        account_name: account_info.account_name,
        channel_handle: account_info.channel_handle,
        account_photo_url: account_info.account_photo_url,
    }
}

pub fn library_artists_page_to_proto(
    page: ytmusicapi::Page<ytmusicapi::LibraryArtist, ytmusicapi::LibraryArtistsContinuationToken>,
) -> pb::LibraryArtistsResponse {
    pb::LibraryArtistsResponse {
        items: page
            .items
            .into_iter()
            .map(library_artist_item_to_proto)
            .collect(),
        continuation_token: page.continuation.map(|token| token.as_str().to_owned()),
    }
}

pub fn library_albums_page_to_proto(
    page: ytmusicapi::Page<ytmusicapi::LibraryAlbum, ytmusicapi::LibraryAlbumsContinuationToken>,
) -> pb::LibraryAlbumsResponse {
    pb::LibraryAlbumsResponse {
        items: page
            .items
            .into_iter()
            .map(library_album_item_to_proto)
            .collect(),
        continuation_token: page.continuation.map(|token| token.as_str().to_owned()),
    }
}

pub fn library_subscriptions_page_to_proto(
    page: ytmusicapi::Page<
        ytmusicapi::LibrarySubscription,
        ytmusicapi::LibrarySubscriptionsContinuationToken,
    >,
) -> pb::LibrarySubscriptionsResponse {
    pb::LibrarySubscriptionsResponse {
        items: page
            .items
            .into_iter()
            .map(library_subscription_item_to_proto)
            .collect(),
        continuation_token: page.continuation.map(|token| token.as_str().to_owned()),
    }
}

pub fn library_channels_page_to_proto(
    page: ytmusicapi::Page<
        ytmusicapi::LibraryChannel,
        ytmusicapi::LibraryChannelsContinuationToken,
    >,
) -> pb::LibraryChannelsResponse {
    pb::LibraryChannelsResponse {
        items: page
            .items
            .into_iter()
            .map(library_channel_item_to_proto)
            .collect(),
        continuation_token: page.continuation.map(|token| token.as_str().to_owned()),
    }
}

pub fn library_podcasts_page_to_proto(
    page: ytmusicapi::Page<
        ytmusicapi::LibraryPodcast,
        ytmusicapi::LibraryPodcastsContinuationToken,
    >,
) -> pb::LibraryPodcastsResponse {
    pb::LibraryPodcastsResponse {
        items: page
            .items
            .into_iter()
            .map(library_podcast_item_to_proto)
            .collect(),
        continuation_token: page.continuation.map(|token| token.as_str().to_owned()),
    }
}

pub fn library_songs_page_to_proto(
    page: ytmusicapi::Page<ytmusicapi::LibrarySong, ytmusicapi::LibrarySongsContinuationToken>,
) -> pb::LibrarySongsResponse {
    pb::LibrarySongsResponse {
        items: page
            .items
            .into_iter()
            .map(library_song_item_to_proto)
            .collect(),
        continuation_token: page.continuation.map(|token| token.as_str().to_owned()),
    }
}

pub fn liked_songs_page_to_proto(page: ytmusicapi::LikedSongsPage) -> pb::LikedSongsResponse {
    pb::LikedSongsResponse {
        playlist_id: page.playlist_id,
        title: page.title,
        items: page
            .items
            .into_iter()
            .map(liked_song_item_to_proto)
            .collect(),
        thumbnails: page
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
        continuation_token: page.continuation.map(|token| token.as_str().to_owned()),
    }
}

pub fn saved_episodes_page_to_proto(
    page: ytmusicapi::SavedEpisodesPage,
) -> pb::SavedEpisodesResponse {
    pb::SavedEpisodesResponse {
        playlist_id: page.playlist_id,
        title: page.title,
        items: page
            .items
            .into_iter()
            .map(saved_episode_item_to_proto)
            .collect(),
        thumbnails: page
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
        continuation_token: page.continuation.map(|token| token.as_str().to_owned()),
    }
}

fn search_result_to_proto(result: ytmusicapi::SearchResult) -> pb::SearchResult {
    let kind = match result {
        ytmusicapi::SearchResult::Song(song) => {
            Some(search_result::Kind::Song(song_search_result_to_proto(song)))
        }
        ytmusicapi::SearchResult::Video(video) => Some(search_result::Kind::Video(
            video_search_result_to_proto(video),
        )),
        ytmusicapi::SearchResult::Episode(episode) => Some(search_result::Kind::Episode(
            video_search_result_to_proto(episode),
        )),
        ytmusicapi::SearchResult::Album(album) => Some(search_result::Kind::Album(
            album_search_result_to_proto(album),
        )),
        ytmusicapi::SearchResult::Artist(artist) => Some(search_result::Kind::Artist(
            artist_search_result_to_proto(artist),
        )),
        ytmusicapi::SearchResult::Profile(profile) => Some(search_result::Kind::Profile(
            profile_search_result_to_proto(profile),
        )),
        ytmusicapi::SearchResult::Playlist(playlist) => Some(search_result::Kind::Playlist(
            playlist_search_result_to_proto(playlist),
        )),
        ytmusicapi::SearchResult::Podcast(podcast) => Some(search_result::Kind::Podcast(
            playlist_search_result_to_proto(podcast),
        )),
    };

    pb::SearchResult { kind }
}

fn song_search_result_to_proto(result: ytmusicapi::SongResult) -> pb::SongSearchResult {
    pb::SongSearchResult {
        category: result.category,
        result_type: search_result_type_to_proto(result.result_type) as i32,
        video_id: result.video_id,
        title: result.title,
        artists: result
            .artists
            .into_iter()
            .map(artist_ref_to_proto)
            .collect(),
        album: result.album.map(album_ref_to_proto),
        duration: result.duration,
        thumbnails: result
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
        is_explicit: result.is_explicit,
    }
}

fn video_search_result_to_proto(result: ytmusicapi::VideoResult) -> pb::VideoSearchResult {
    pb::VideoSearchResult {
        category: result.category,
        result_type: search_result_type_to_proto(result.result_type) as i32,
        title: result.title,
        video_id: result.video_id,
        video_type: result.video_type,
        artists: result
            .artists
            .into_iter()
            .map(artist_ref_to_proto)
            .collect(),
        thumbnails: result
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
        duration: result.duration,
        views: result.views,
        date: result.date,
        podcast: result.podcast.map(album_ref_to_proto),
        live: result.live,
    }
}

fn album_search_result_to_proto(result: ytmusicapi::AlbumResult) -> pb::AlbumSearchResult {
    pb::AlbumSearchResult {
        category: result.category,
        result_type: search_result_type_to_proto(result.result_type) as i32,
        browse_id: result.browse_id,
        playlist_id: result.playlist_id,
        title: result.title,
        type_label: result.type_label,
        year: result.year,
        duration: result.duration,
        is_explicit: result.is_explicit,
        artists: result
            .artists
            .into_iter()
            .map(artist_ref_to_proto)
            .collect(),
        thumbnails: result
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
    }
}

fn artist_search_result_to_proto(result: ytmusicapi::ArtistResult) -> pb::ArtistSearchResult {
    pb::ArtistSearchResult {
        category: result.category,
        result_type: search_result_type_to_proto(result.result_type) as i32,
        artist: result.artist,
        artists: result
            .artists
            .into_iter()
            .map(artist_ref_to_proto)
            .collect(),
        subscribers: result.subscribers,
        browse_id: result.browse_id,
        radio_id: result.radio_id,
        shuffle_id: result.shuffle_id,
        thumbnails: result
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
    }
}

fn profile_search_result_to_proto(result: ytmusicapi::ProfileResult) -> pb::ProfileSearchResult {
    pb::ProfileSearchResult {
        category: result.category,
        result_type: search_result_type_to_proto(result.result_type) as i32,
        browse_id: result.browse_id,
        name: result.name,
        handle: result.handle,
        thumbnails: result
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
    }
}

fn playlist_search_result_to_proto(result: ytmusicapi::PlaylistResult) -> pb::PlaylistSearchResult {
    pb::PlaylistSearchResult {
        category: result.category,
        result_type: search_result_type_to_proto(result.result_type) as i32,
        browse_id: result.browse_id,
        title: result.title,
        author: result.author,
        item_count: result.item_count,
        thumbnails: result
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
    }
}

fn watch_track_to_proto(track: ytmusicapi::WatchTrack) -> pb::WatchTrackItem {
    pb::WatchTrackItem {
        video_id: track.video_id,
        title: track.title,
        duration: track.duration,
        thumbnails: track
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
        artists: track.artists.into_iter().map(artist_ref_to_proto).collect(),
        album: track.album.map(album_ref_to_proto),
        like_status: track
            .like_status
            .map(|status| like_status_to_proto(status) as i32),
        video_type: track.video_type,
        year: track.year,
        views: track.views,
        is_in_library: track.is_in_library,
        counterpart: track
            .counterpart
            .map(|counterpart| Box::new(watch_track_to_proto(*counterpart))),
    }
}

fn library_playlist_item_to_proto(item: ytmusicapi::LibraryPlaylist) -> pb::LibraryPlaylistItem {
    pb::LibraryPlaylistItem {
        playlist_id: item.playlist_id,
        title: item.title,
        authors: item.authors.into_iter().map(artist_ref_to_proto).collect(),
        item_count: item.item_count,
        thumbnails: item
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
    }
}

fn library_artist_item_to_proto(item: ytmusicapi::LibraryArtist) -> pb::LibraryArtistItem {
    pb::LibraryArtistItem {
        browse_id: item.browse_id,
        artist: item.artist,
        subscribers: item.subscribers,
        thumbnails: item
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
    }
}

fn library_album_item_to_proto(item: ytmusicapi::LibraryAlbum) -> pb::LibraryAlbumItem {
    pb::LibraryAlbumItem {
        browse_id: item.browse_id,
        playlist_id: item.playlist_id,
        title: item.title,
        type_label: item.type_label,
        artists: item.artists.into_iter().map(artist_ref_to_proto).collect(),
        year: item.year,
        thumbnails: item
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
    }
}

fn library_subscription_item_to_proto(
    item: ytmusicapi::LibrarySubscription,
) -> pb::LibrarySubscriptionItem {
    pb::LibrarySubscriptionItem {
        browse_id: item.browse_id,
        name: item.name,
        subscribers: item.subscribers,
        thumbnails: item
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
    }
}

fn library_channel_item_to_proto(item: ytmusicapi::LibraryChannel) -> pb::LibraryChannelItem {
    pb::LibraryChannelItem {
        browse_id: item.browse_id,
        name: item.name,
        subscribers: item.subscribers,
        thumbnails: item
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
    }
}

fn library_podcast_item_to_proto(item: ytmusicapi::LibraryPodcast) -> pb::LibraryPodcastItem {
    pb::LibraryPodcastItem {
        title: item.title,
        browse_id: item.browse_id,
        podcast_id: item.podcast_id,
        channel: Some(library_podcast_channel_to_proto(item.channel)),
        thumbnails: item
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
    }
}

fn library_podcast_channel_to_proto(
    channel: ytmusicapi::LibraryPodcastChannel,
) -> pb::LibraryPodcastChannel {
    pb::LibraryPodcastChannel {
        id: channel.id,
        name: channel.name,
    }
}

fn library_song_item_to_proto(item: ytmusicapi::LibrarySong) -> pb::LibrarySongItem {
    pb::LibrarySongItem {
        video_id: item.video_id,
        title: item.title,
        artists: item.artists.into_iter().map(artist_ref_to_proto).collect(),
        album: item.album.map(album_ref_to_proto),
        duration: item.duration,
        thumbnails: item
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
        like_status: item
            .like_status
            .map(|status| like_status_to_proto(status) as i32),
    }
}

fn liked_song_item_to_proto(item: ytmusicapi::LikedSongItem) -> pb::LikedSongItem {
    pb::LikedSongItem {
        video_id: item.video_id,
        title: item.title,
        artists: item.artists.into_iter().map(artist_ref_to_proto).collect(),
        album: item.album.map(album_ref_to_proto),
        duration: item.duration,
        thumbnails: item
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
        like_status: item
            .like_status
            .map(|status| like_status_to_proto(status) as i32),
    }
}

fn saved_episode_item_to_proto(item: ytmusicapi::SavedEpisodeItem) -> pb::SavedEpisodeItem {
    pb::SavedEpisodeItem {
        video_id: item.video_id,
        title: item.title,
        channel: item.channel,
        podcast: item.podcast,
        duration: item.duration,
        thumbnails: item
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
    }
}

fn song_video_details_to_proto(details: ytmusicapi::SongVideoDetails) -> pb::SongVideoDetails {
    pb::SongVideoDetails {
        video_id: details.video_id,
        title: details.title,
        length_seconds: details.length_seconds,
        channel_id: details.channel_id,
        author: details.author,
        thumbnails: details
            .thumbnails
            .into_iter()
            .map(thumbnail_to_proto)
            .collect(),
        allow_ratings: details.allow_ratings,
        view_count: details.view_count,
        is_owner_viewing: details.is_owner_viewing,
        is_crawlable: details.is_crawlable,
        is_private: details.is_private,
        is_unplugged_corpus: details.is_unplugged_corpus,
        is_live_content: details.is_live_content,
        is_tvfilm_video: details.is_tvfilm_video,
        music_video_type: details.music_video_type,
    }
}

fn song_playability_status_to_proto(
    status: ytmusicapi::SongPlayabilityStatus,
) -> pb::SongPlayabilityStatus {
    pb::SongPlayabilityStatus {
        status: status.status,
        playable_in_embed: status.playable_in_embed,
        reason: status.reason,
        context_params: status.context_params,
        audio_only_availability: status.audio_only_availability,
        playback_mode: status.playback_mode,
    }
}

fn song_streaming_data_to_proto(data: ytmusicapi::SongStreamingData) -> pb::SongStreamingData {
    pb::SongStreamingData {
        expires_in_seconds: data.expires_in_seconds,
        server_abr_streaming_url: data.server_abr_streaming_url,
        formats: data
            .formats
            .into_iter()
            .map(song_stream_format_to_proto)
            .collect(),
        adaptive_formats: data
            .adaptive_formats
            .into_iter()
            .map(song_stream_format_to_proto)
            .collect(),
    }
}

fn song_stream_format_to_proto(format: ytmusicapi::SongStreamFormat) -> pb::SongStreamFormat {
    pb::SongStreamFormat {
        itag: format.itag,
        mime_type: format.mime_type,
        bitrate: format.bitrate,
        average_bitrate: format.average_bitrate,
        content_length: format.content_length,
        last_modified: format.last_modified,
        quality: format.quality,
        quality_label: format.quality_label,
        quality_ordinal: format.quality_ordinal,
        projection_type: format.projection_type,
        width: format.width,
        height: format.height,
        fps: format.fps,
        color_info: format.color_info.map(song_color_info_to_proto),
        audio_quality: format.audio_quality,
        audio_sample_rate: format.audio_sample_rate,
        audio_channels: format.audio_channels,
        loudness_db: format.loudness_db,
        track_absolute_loudness_lkfs: format.track_absolute_loudness_lkfs,
        approx_duration_ms: format.approx_duration_ms,
        high_replication: format.high_replication,
        xtags: format.xtags,
        init_range: format.init_range.map(song_byte_range_to_proto),
        index_range: format.index_range.map(song_byte_range_to_proto),
        signature_cipher: format.signature_cipher,
    }
}

fn song_byte_range_to_proto(range: ytmusicapi::SongByteRange) -> pb::SongByteRange {
    pb::SongByteRange {
        start: range.start,
        end: range.end,
    }
}

fn song_color_info_to_proto(info: ytmusicapi::SongColorInfo) -> pb::SongColorInfo {
    pb::SongColorInfo {
        primaries: info.primaries,
        transfer_characteristics: info.transfer_characteristics,
        matrix_coefficients: info.matrix_coefficients,
    }
}

fn song_microformat_to_proto(microformat: ytmusicapi::SongMicroformat) -> pb::SongMicroformat {
    pb::SongMicroformat {
        url_canonical: microformat.url_canonical,
        description: microformat.description,
        category: microformat.category,
        publish_date: microformat.publish_date,
        upload_date: microformat.upload_date,
        view_count: microformat.view_count,
        available_countries: microformat.available_countries,
        tags: microformat.tags,
        noindex: microformat.noindex,
        unlisted: microformat.unlisted,
        family_safe: microformat.family_safe,
    }
}

fn thumbnail_to_proto(thumbnail: ytmusicapi::Thumbnail) -> pb::Thumbnail {
    pb::Thumbnail {
        url: thumbnail.url,
        width: thumbnail.width,
        height: thumbnail.height,
    }
}

fn artist_ref_to_proto(artist: ytmusicapi::ArtistRef) -> pb::ArtistRef {
    pb::ArtistRef {
        id: artist.id,
        name: artist.name,
    }
}

fn album_ref_to_proto(album: ytmusicapi::AlbumRef) -> pb::AlbumRef {
    pb::AlbumRef {
        id: album.id,
        name: album.name,
    }
}

fn search_result_type_to_proto(result_type: ytmusicapi::SearchResultType) -> pb::SearchResultType {
    match result_type {
        ytmusicapi::SearchResultType::Song => pb::SearchResultType::Song,
        ytmusicapi::SearchResultType::Video => pb::SearchResultType::Video,
        ytmusicapi::SearchResultType::Album => pb::SearchResultType::Album,
        ytmusicapi::SearchResultType::Artist => pb::SearchResultType::Artist,
        ytmusicapi::SearchResultType::Profile => pb::SearchResultType::Profile,
        ytmusicapi::SearchResultType::Playlist => pb::SearchResultType::Playlist,
        ytmusicapi::SearchResultType::Episode => pb::SearchResultType::Episode,
        ytmusicapi::SearchResultType::Podcast => pb::SearchResultType::Podcast,
    }
}

fn like_status_to_proto(status: ytmusicapi::LibraryLikeStatus) -> pb::LibraryLikeStatus {
    match status {
        ytmusicapi::LibraryLikeStatus::Like => pb::LibraryLikeStatus::Like,
        ytmusicapi::LibraryLikeStatus::Indifferent => pb::LibraryLikeStatus::Indifferent,
        ytmusicapi::LibraryLikeStatus::Dislike => pb::LibraryLikeStatus::Dislike,
    }
}

#[cfg(test)]
mod tests {
    use super::{library_podcasts_page_to_proto, saved_episodes_page_to_proto};

    fn thumbnail(url: &str, width: u32, height: u32) -> ytmusicapi::Thumbnail {
        ytmusicapi::Thumbnail {
            url: url.to_owned(),
            width,
            height,
        }
    }

    #[test]
    fn library_podcasts_page_to_proto_preserves_nested_channel_fields() {
        let proto = library_podcasts_page_to_proto(ytmusicapi::Page {
            items: vec![ytmusicapi::LibraryPodcast {
                title: "On Air".to_owned(),
                browse_id: "browse-1".to_owned(),
                podcast_id: "podcast-1".to_owned(),
                channel: ytmusicapi::LibraryPodcastChannel {
                    id: Some("channel-42".to_owned()),
                    name: "Waveform".to_owned(),
                },
                thumbnails: vec![thumbnail("https://example.com/podcast.jpg", 320, 320)],
            }],
            continuation: Some(ytmusicapi::LibraryPodcastsContinuationToken::new(
                "next-podcast",
            )),
        });

        assert_eq!(proto.items.len(), 1);
        assert_eq!(proto.items[0].title, "On Air");
        assert_eq!(proto.items[0].browse_id, "browse-1");
        assert_eq!(proto.items[0].podcast_id, "podcast-1");
        assert_eq!(
            proto.items[0]
                .channel
                .as_ref()
                .and_then(|channel| channel.id.as_deref()),
            Some("channel-42")
        );
        assert_eq!(
            proto.items[0]
                .channel
                .as_ref()
                .map(|channel| channel.name.as_str()),
            Some("Waveform")
        );
        assert_eq!(proto.items[0].thumbnails.len(), 1);
        assert_eq!(
            proto.items[0].thumbnails[0].url,
            "https://example.com/podcast.jpg"
        );
        assert_eq!(proto.continuation_token.as_deref(), Some("next-podcast"));
    }

    #[test]
    fn saved_episodes_page_to_proto_preserves_episode_fields() {
        let proto = saved_episodes_page_to_proto(ytmusicapi::SavedEpisodesPage {
            playlist_id: "SE".to_owned(),
            title: "Saved Episodes".to_owned(),
            items: vec![ytmusicapi::SavedEpisodeItem {
                video_id: "episode-7".to_owned(),
                title: "Episode Seven".to_owned(),
                channel: "Waveform".to_owned(),
                podcast: "On Air".to_owned(),
                duration: Some("42:00".to_owned()),
                thumbnails: vec![thumbnail("https://example.com/episode.jpg", 480, 480)],
            }],
            thumbnails: vec![thumbnail("https://example.com/saved.jpg", 800, 800)],
            continuation: Some(ytmusicapi::SavedEpisodesContinuationToken::new(
                "saved-next",
            )),
        });

        assert_eq!(proto.playlist_id, "SE");
        assert_eq!(proto.title, "Saved Episodes");
        assert_eq!(proto.items.len(), 1);
        assert_eq!(proto.items[0].video_id, "episode-7");
        assert_eq!(proto.items[0].title, "Episode Seven");
        assert_eq!(proto.items[0].channel, "Waveform");
        assert_eq!(proto.items[0].podcast, "On Air");
        assert_eq!(proto.items[0].duration.as_deref(), Some("42:00"));
        assert_eq!(proto.items[0].thumbnails.len(), 1);
        assert_eq!(
            proto.items[0].thumbnails[0].url,
            "https://example.com/episode.jpg"
        );
        assert_eq!(proto.thumbnails.len(), 1);
        assert_eq!(proto.thumbnails[0].url, "https://example.com/saved.jpg");
        assert_eq!(proto.continuation_token.as_deref(), Some("saved-next"));
    }
}
