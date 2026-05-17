# ytmusic-service API Reference

The API uses two gRPC package namespaces: `ytmusic.v1` for the public service and `ytmusic.v1.admin` for the admin service.

## Service names

- `ytmusic.v1.YtMusicPublic`
- `ytmusic.v1.admin.YtMusicAdmin`

The admin listener is also the endpoint used for reflection-backed `grpcurl describe` and `grpcurl list` queries.

## Public API summary

### Search and discovery

- `Search` searches the catalog and account-visible content.
- `SearchContinuation` continues a prior search result set with a continuation token.

### Watch playlist and playback metadata

- `GetWatchPlaylist` resolves the watch playlist for a video and optional playlist playback context.
- `GetWatchPlaylistContinuation` continues a watch playlist response with a continuation token.
- `GetSong` returns song metadata for a single video ID.
- `Decipher` turns a `signature_cipher` value into a playable URL.

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

## Admin API summary

- `ReloadBrowserAuth` reloads `browser.json` from the configured path and swaps the in-memory auth state if validation succeeds.

## Proto sources

- [`public.proto`](../proto/ytmusic/v1/public.proto)
- [`admin.proto`](../proto/ytmusic/v1/admin.proto)
