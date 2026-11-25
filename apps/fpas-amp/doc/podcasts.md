# Podcasts

Open **Podcasts → Search** to browse up to 100 shows in the Apple Podcasts
charts. Open **Filters** to choose a market and an Apple genre, then select
**Apply**. The market defaults to your system region. Each market lists the
shows available there. **Reset** restores the default market and all genres in
the filter form. **Subscription** limits the returned shows to those you have
subscribed to or those you have yet to subscribe to.

Submit a name or publisher to search the selected market. Changing the market
reruns the submitted search while preserving any edits in the search field.
**Genre** narrows
the loaded text matches and the status bar shows how many remain. Clear the
search field to return to the charts with the same filters. Choose a show to
open its episode list, or click **Subscribe** to add it below **Search** in the
sidebar. Use **Add RSS feed** to enter a feed address directly. That flow loads
the feed before saving the subscription. Use the sort menu or the **Podcast**
and **Publisher** headings to sort the results. Choose **Apple order** in the
menu to restore the directory order.

Select a subscription to load episodes from its publisher. Search within the
episode list and use **Filters** to narrow it by publication year or duration.
The year picker appears when the episodes span multiple years or mix dated and
undated entries. Choose **Unknown year** to see the undated episodes. A feed
refresh clears a year selection if that choice disappears.
Use the sort menu or a column heading to sort by title, publication date or
duration. Episodes start newest first. Unknown dates and durations sort last
in either direction. Both tables build rows as they come into view. Open **Episode details**
from the row's three-dot menu to see its description. **Unsubscribe** removes the sidebar
entry. Subscriptions are saved locally in `settings.json`.

Playback uses the main player with its usual controls, equalizer and waveform.
The app downloads the selected enclosure to a private temporary directory
before opening it. Complete downloads can be reused during the same session.
The directory is removed on normal exit. The player can pause, seek and move
through the episodes from the selected list. Episode playback position is
held for the current session. Starting an episode captures the table's filtered
order for playback.

## Implementation

Discovery loads the overall or category chart for the chosen market and
resolves up to 100 ranked shows to public RSS addresses in a single lookup.
Text search matches the selected genre against Apple's genre IDs in the
results. The requests use no API key. Requests to the iTunes host start at
least 3.2 seconds apart. RSS 2.0 supplies
the episode metadata and direct audio enclosure URLs. Larger media responses
are limited to 512 MiB. Search and feed replies carry request identities, and
changing shows cancels the previous feed request. The typed search and feed
effects deliver metadata before artwork. The download effect reports either a
complete path or an expected download failure. Invalid directory response
formats are reported as search failures. Unusable chart rows are skipped.

- [Apple Search API](https://performance-partners.apple.com/search-api)
- [Apple Podcasts Charts](https://podcasters.apple.com/support/3146-apple-podcasts-charts)
- [Apple Podcast Categories](https://podcasters.apple.com/support/1691-apple-podcasts-categories)
- [Apple RSS Feed Builder](https://rss.marketingtools.apple.com/)
- [RSS 2.0 specification](https://www.rssboard.org/rss-specification)

Sources

- [Model.fpas](../src/FpasAmp.Ui/Podcasts/Model.fpas)
- [Browse.fpas](../src/FpasAmp.Ui/Podcasts/Browse.fpas)
- [Catalog.fpas](../src/FpasAmp.Ui/Podcasts/Catalog.fpas)
- [Directory.fpas](../src/FpasAmp.Ui/Podcasts/Directory.fpas)
- [DirectoryData.fpas](../src/FpasAmp.Ui/Podcasts/DirectoryData.fpas)
- [Toolbar.fpas](../src/FpasAmp.Ui/Podcasts/Toolbar.fpas)
- [Feeds.fpas](../src/FpasAmp.Ui/Podcasts/Feeds.fpas)
- [Network.fpas](../src/FpasAmp.Ui/Podcasts/Network.fpas)
- [Downloads.fpas](../src/FpasAmp.Ui/Podcasts/Downloads.fpas)
- [Update.fpas](../src/FpasAmp.Ui/Podcasts/Update.fpas)
- [View.fpas](../src/FpasAmp.Ui/Podcasts/View.fpas)
- [Podcasts.fpas](../src/FpasAmp.Ui/App/Podcasts.fpas)
- [PodcastSubscriptions.fpas](../src/FpasAmp/Persistence/PodcastSubscriptions.fpas)

## Verification

The headless effect test holds a loopback feed's metadata, cover, and malformed
XML failure separately and checks the model before and after each reply. It
also verifies the submitted search and the provider response parser.

The native test releases directory, RSS, cover and downloaded audio replies
individually. It checks the charts before typing, after changing market and
genre, and after clearing the query. It also checks genre filtering on the
loaded search results, the subscription entry, saved route, finite audio
decoder, waveform, seek and next episode.

- [NativePodcasts.fpas](../tests/FpasAmp.Ui.Tests/Providers/NativePodcasts.fpas)
- [PodcastEffects.fpas](../tests/FpasAmp.Ui.Tests/Providers/PodcastEffects.fpas)
- [ProviderHttp.fpas](../tests/FpasAmp.Ui.Tests/Providers/ProviderHttp.fpas)
