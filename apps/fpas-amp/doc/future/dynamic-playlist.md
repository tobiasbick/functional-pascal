# Dynamic playlists for FPas-Amp

The [tag editor](../desktop.md#editing-track-tags) supports MusicBrainz search
and local tag edits. Dynamic playback and background lookup are pending.

Start a dynamic playlist from a selected or playing track. Use similar recordings
from ListenBrainz to choose what plays next from the local library. If the online
search yields too few playable tracks, use local matches by genre and artist.

## AcoustID in the tag editor

- [ ] Enable **Submit fingerprint** when both keys are available and the user has confirmed
  the MusicBrainz recording match. Use the keys saved in [Settings → Metadata](../desktop.md#settings-and-scan-control)
  for `client` and `user`. Make submission a separate action in the
  editor. Send the Chromaprint fingerprint, full track duration and confirmed
  recording MBID to AcoustID on that action. The audio file stays local.
  [AcoustID submission API](https://acoustid.org/webservice#submit)
- [ ] Retain the returned submission ID. Show **Submitted** when AcoustID
  accepts the request and **Imported** when its status confirms the import.
  [AcoustID submission status](https://acoustid.org/webservice#get-submission-status)

## Resolve the recording ID

ListenBrainz's similar-recordings endpoint takes a MusicBrainz recording ID
(MBID). The [library](../library.md#acoustic-fingerprints) already caches acoustic
fingerprints, and the metadata reader already reads recording IDs from file
tags. A fingerprint identifies audio for a lookup. MusicBrainz assigns the
recording ID. [MusicBrainz identifiers](https://musicbrainz.org/doc/MusicBrainz_Identifier)

1. Use the recording MBID from the tags or reuse a confirmed match in `library.db`.
2. For a missing ID, query MusicBrainz by ISRC when available, or by track title
   and artist. Check the album, duration and track number where available.
   For a complete, reliably tagged album, resolve the release, fetch its
   recordings together and match the track list locally. Keep ambiguous studio,
   live and remix matches unresolved. [Recording search](https://musicbrainz.org/doc/MusicBrainz_API/Search/RecordingSearch) · [MusicBrainz API](https://musicbrainz.org/doc/MusicBrainz_API)
3. Evaluate ListenBrainz Labs `acr-lookup` and `acrr-lookup` as a second way to
   turn title and artist, with the album where known, into a recording MBID.
   A sample `acrr-lookup` request succeeded without credentials. The main ListenBrainz
   `/1/metadata/lookup/` endpoints require a user token. [Labs lookup](https://labs.api.listenbrainz.org/acrr-lookup) · [ListenBrainz metadata API](https://listenbrainz.readthedocs.io/en/latest/users/api/metadata.html#get--1-metadata-lookup-)
4. If the tags are incomplete or the match is uncertain and the user has
   configured an AcoustID application key, use fingerprint lookup. Reuse the
   cached Chromaprint fingerprint or generate it locally with `fpcalc`. Send
   the fingerprint and full track duration with `meta=recordingids`. Accept an
   unambiguous recording MBID. [AcoustID lookup](https://acoustid.org/webservice#lookup-by-fingerprint)

Store confirmed IDs and their source in `library.db`. Open audio files in
read-only mode during lookup. Keep the lookup status separate from the metadata
read from the file so another scan can reuse the result.

## Background lookup after a scan

- [ ] When the inventory and tag passes settle, enqueue available tracks with
  missing recording IDs. Use one low-priority worker and give a playing track
  or a requested playlist seed priority over the backlog. Reuse completed
  fingerprint work. Leave paused or offline sources out of background work.
- [ ] Save progress so the job can pause, resume and continue after an app
  restart. Deduplicate pending requests. Drop a result if the indexed file
  changed while it was in flight.
- [ ] Cache confirmed matches and record inconclusive or empty results so a
  routine rescan does not repeat them. Retry temporary network and server
  failures with backoff. Keep the scan and playback usable while retries wait.

## Provider access and pacing

| Service | Access | Rate limit and planned pacing |
| --- | --- | --- |
| MusicBrainz | Public read requests work without an account, app registration or API key. Send an identifying `User-Agent` with the app name, version and project contact URL. | At most one request per second per client. MusicBrainz also meters the source IP. [API rate limits and identification](https://musicbrainz.org/doc/MusicBrainz_API/Rate_Limiting) |
| AcoustID | The user supplies an application key for lookup. Submission also uses their personal User API key. Noncommercial use is free. Contact AcoustID before a rollout likely to generate significant traffic. | At most three requests per second. Start the background job at one lookup per second. [API guidelines](https://acoustid.org/webservice#usage-guidelines) |
| ListenBrainz Labs | Similar-recordings and the Labs metadata lookup worked without credentials in sample requests. | Start at no more than one request per second as an app policy. Use the same identifying `User-Agent`. Honor any rate-limit headers the Labs host sends and back off on throttling. The [published ListenBrainz API limits](https://listenbrainz.readthedocs.io/en/latest/users/api/index.html#rate-limiting) describe the main API. |

```http
User-Agent: FPas-Amp/<version> (https://github.com/tobiasbick/functional-pascal/issues)
```

Share each provider's rate limiter across background and on-demand work, including
AcoustID submissions and status checks. At one lookup per second, 22,000 missing
IDs take about six hours if each needs exactly one request. Allow for fingerprint
calculation, retries and additional lookups.

## Build the playback queue

- [ ] Resolve the seed track on demand. Query ListenBrainz Labs
  `similar-recordings` with its MBID and cache suggestions for reuse.
  [Similar Recordings Viewer](https://labs.api.listenbrainz.org/similar-recordings)
- [ ] Match recommendations to playable local files by recording MBID where
  available. Use title and artist for a unique local match when the recommended
  track has no local MBID yet. This lets the first playlist run while the
  background job is still working.
- [ ] Add a small batch of matches to the playback queue and refill it as it
  runs low. Skip the starting track and recent repeats. Fill gaps from the local
  genre and artist match when the connection is unavailable, the seed cannot
  be resolved or the suggestions are absent from the library.

## Verification before release

- [ ] Test reuse of embedded IDs, persisted matches and cached fingerprints.
  Cover ambiguous recordings, no match and reopening with work still pending.
- [ ] Verify pacing with a controlled clock and HTTP responses for throttling,
  transient errors and outages. Keep these tests independent of the live
  services.
- [ ] Verify that the application key enables lookup on its own, and that
  submission requires both keys and a confirmed match. Check the transition
  from **Submitted** to **Imported** with controlled API responses.
- [ ] Use `Fpas.Ui.Mvu.Testing.EffectReplies` to release delayed lookup and
  recommendation replies individually. Check the visible queue before and
  after each reply when the user changes tracks or starts a different mix.

See the [desktop playback guide](../desktop.md#playback-and-analysis) for the
existing queue and audio behavior.
