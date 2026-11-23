# Podcasts

Open **Podcasts → Search** to browse up to 100 shows in the Apple Podcasts
charts. **Market** starts at your system region and can be changed. Each
market lists the shows available there. **Genre** selects a chart for one of
Apple's main podcast categories.

Submit a name or publisher to search the selected market. **Genre** narrows
the loaded text matches and the status bar shows how many remain. Clear the
search field to return to the charts with the same filters. Choose a show to
open its episode list, or click **Subscribe** to add it below **Search** in the
sidebar. Use **Add RSS feed** to enter a feed address directly. That flow loads
the feed before saving the subscription.

Select a subscription to load episodes from its publisher. Search within the
episode list, change its date order or play an episode. Open **Episode details**
from the row's three-dot menu to see its description. **Unsubscribe** removes the sidebar
entry. Subscriptions are saved locally in `settings.json`.

Playback uses the main player with its usual controls, equalizer and waveform.
The app downloads the selected enclosure to a private temporary directory
before opening it. Complete downloads can be reused during the same session.
The directory is removed on normal exit. The player can pause, seek and move
through the episodes from the selected list. Episode playback position is
held for the current session.

## Implementation

Discovery loads the overall or category chart for the chosen market and
resolves up to 100 ranked shows to public RSS addresses in a single lookup.
Text search matches the selected genre against Apple's genre IDs in the
results. The requests use no API key. Requests to the iTunes host start at
least 3.2 seconds apart. RSS 2.0 supplies
the episode metadata and direct audio enclosure URLs. Larger media responses
are limited to 512 MiB. Search and feed replies carry request identities, and
changing shows cancels the previous feed request.

- [Apple Search API](https://performance-partners.apple.com/search-api)
- [Apple Podcasts Charts](https://podcasters.apple.com/support/3146-apple-podcasts-charts)
- [Apple Podcast Categories](https://podcasters.apple.com/support/1691-apple-podcasts-categories)
- [Apple RSS Feed Builder](https://rss.marketingtools.apple.com/)
- [RSS 2.0 specification](https://www.rssboard.org/rss-specification)

Sources

- [Model.fpas](../src/FpasAmp.Ui/Podcasts/Model.fpas)
- [Catalog.fpas](../src/FpasAmp.Ui/Podcasts/Catalog.fpas)
- [Directory.fpas](../src/FpasAmp.Ui/Podcasts/Directory.fpas)
- [DiscoveryView.fpas](../src/FpasAmp.Ui/Podcasts/DiscoveryView.fpas)
- [Feeds.fpas](../src/FpasAmp.Ui/Podcasts/Feeds.fpas)
- [Network.fpas](../src/FpasAmp.Ui/Podcasts/Network.fpas)
- [Downloads.fpas](../src/FpasAmp.Ui/Podcasts/Downloads.fpas)
- [Update.fpas](../src/FpasAmp.Ui/Podcasts/Update.fpas)
- [View.fpas](../src/FpasAmp.Ui/Podcasts/View.fpas)
- [Podcasts.fpas](../src/FpasAmp.Ui/App/Podcasts.fpas)
- [PodcastSubscriptions.fpas](../src/FpasAmp/Persistence/PodcastSubscriptions.fpas)

## Verification

The native test releases directory, RSS, cover and downloaded audio replies
individually. It checks the charts before typing, after changing market and
genre, and after clearing the query. It also checks genre filtering on the
loaded search results, the subscription entry, saved route, finite audio
decoder, waveform, seek and next episode.

- [NativePodcasts.fpas](../tests/FpasAmp.Ui.Tests/NativePodcasts.fpas)
- [ProviderHttp.fpas](../tests/FpasAmp.Ui.Tests/ProviderHttp.fpas)
