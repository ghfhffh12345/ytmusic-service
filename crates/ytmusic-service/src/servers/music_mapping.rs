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
