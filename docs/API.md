# ytmusic-service API Reference

The service exposes two gRPC packages: `ytmusic.v1` on the public listener and `ytmusic.v1.admin` on the admin listener.

## Service names

- `ytmusic.v1.YtMusicPublic`
- `ytmusic.v1.admin.YtMusicAdmin`

The admin listener is also the endpoint used for reflection-backed `grpcurl describe` and `grpcurl list` queries.

## Public API summary

### Search and discovery

- `Search` searches the catalog and account-visible content.
- `SearchContinuation` continues a prior search result set with a continuation token.

### Watch playlist and playback metadata

- `GetWatchPlaylist` resolves the watch playlist for a track, video, or playback context.
- `GetWatchPlaylistContinuation` continues a watch playlist response with a continuation token.
- `GetSong` returns song metadata for a single video ID.
- `Decipher` resolves playback-related values that require deciphering.

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

- `ReloadBrowserAuth` reloads browser-backed authentication state for the service.

## Proto sources

- [`public.proto`](../proto/ytmusic/v1/public.proto)
- [`admin.proto`](../proto/ytmusic/v1/admin.proto)
