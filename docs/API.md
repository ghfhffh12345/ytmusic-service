# ytmusic-service API Reference

For setup, authentication bootstrap, runtime examples, and troubleshooting, start with [README.md](../README.md).

`ytmusic-service` exposes a single gRPC listener with the `ytmusic.v2` service surface plus health and reflection.

## Service names

- `ytmusic.v2.YtMusic`
- `ytmusic.v2.YtCipher`
- `ytmusic.v2.ServiceStatus`
- `grpc.health.v1.Health`
- `grpc.reflection.v1.ServerReflection`

## Public API summary

### Search and discovery

- `Search` searches the catalog and account-visible content.
- `SearchContinuation` continues a prior search result set with a continuation token.

### Watch playlist and playback metadata

- `GetWatchPlaylist` resolves the watch playlist for a `video_id` or `playlist_id`, with optional radio/shuffle playback context.
- `GetWatchPlaylistContinuation` continues a watch playlist response with a continuation token.
- `GetSong` returns song metadata for a single video ID.

### Library listing families

- `GetLibraryPlaylists` and `GetLibraryPlaylistsContinuation`
- `GetLibraryArtists` and `GetLibraryArtistsContinuation`
- `GetLibraryAlbums` and `GetLibraryAlbumsContinuation`
- `GetLibrarySubscriptions` and `GetLibrarySubscriptionsContinuation`
- `GetLibraryChannels` and `GetLibraryChannelsContinuation`
- `GetLibraryPodcasts` and `GetLibraryPodcastsContinuation`
- `GetLibrarySongs` and `GetLibrarySongsContinuation`
- `GetLikedSongs` and `GetLikedSongsContinuation`
- `GetSavedEpisodes` and `GetSavedEpisodesContinuation`

Continuation RPCs consume tokens returned by the corresponding listing call.

### Account information

- `GetAccountInfo` returns account-level profile information.

## Cipher API summary

- `GetSignatureTimestamp` returns the current cipher signature timestamp used for song playback metadata.
- `Refresh` rebuilds the live cipher state.
- `Decipher` turns a `signature_cipher` value into a playable URL.

## Service status API summary

- `GetStatus` reports listener identity, startup time, and subsystem readiness.

## Proto sources

Rust callers can depend on `ytmusic-service-proto` for the generated gRPC types and client/server modules.

- [`music.proto`](../crates/ytmusic-service-proto/proto/ytmusic/v2/music.proto)
- [`cipher.proto`](../crates/ytmusic-service-proto/proto/ytmusic/v2/cipher.proto)
- [`status.proto`](../crates/ytmusic-service-proto/proto/ytmusic/v2/status.proto)
