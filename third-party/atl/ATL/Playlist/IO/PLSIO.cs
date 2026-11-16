using System.IO;
using System.Text;
using System.Collections.Generic;
using Commons;

namespace ATL.Playlist.IO
{
    /// <summary>
    /// PLS playlist manager
    /// </summary>
    public class PLSIO : PlaylistIO
    {
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="filePath">Path of playlist file to load</param>
        /// <param name="readTrackMetadata">Whether to open referenced media.</param>
        public PLSIO(string filePath, bool readTrackMetadata = true) : base(filePath, readTrackMetadata: readTrackMetadata)
        {
        }

        /// <inheritdoc/>
        protected override void load(FileStream fs, IList<FileLocation> locations, IList<Track> tracks)
        {
            Encoding encoding = StreamUtils.GetEncodingFromFileBOM(fs);

            var entries = new SortedDictionary<int, (string path, string title)>();

            using TextReader source = new StreamReader(fs, encoding);
            string s = source.ReadLine();
            while (s != null)
            {
                var parts = s.Split('=', 2);
                if (parts.Length > 1)
                {
                    var index = Utils.ParseFirstIntegerPart(parts[0]);
                    if (index > 0)
                    {
                        entries.TryGetValue(index, out var entry);
                        if (parts[0].StartsWith("file", System.StringComparison.OrdinalIgnoreCase))
                            entries[index] = (parts[1], entry.title ?? "");
                        else if (parts[0].StartsWith("title", System.StringComparison.OrdinalIgnoreCase))
                            entries[index] = (entry.path, parts[1]);
                    }
                }

                s = source.ReadLine();
            }
            foreach (var entry in entries.Values)
                if (!string.IsNullOrEmpty(entry.path))
                    addTrack(decodeLocation(entry.path), entry.title ?? "", locations, tracks);
        }

        /// <inheritdoc/>
        protected override void save(FileStream fs, IList<Track> tracks)
        {
            Encoding encoding = UTF8_NO_BOM;

            using TextWriter w = new StreamWriter(fs, encoding);
            w.WriteLine("[playlist]");

            int counter = 1;
            foreach (Track t in tracks)
            {
                string label = "";
                if (!string.IsNullOrEmpty(t.Title)) label = t.Title;
                if (0 == label.Length) label = System.IO.Path.GetFileNameWithoutExtension(t.Path);

                w.WriteLine("");

                w.Write("File");
                w.Write(counter);
                w.Write("=");
                w.WriteLine(encodeLocation(t.Path)); // Can be rooted or not

                w.Write("Title");
                w.Write(counter);
                w.Write("=");
                w.WriteLine(label);

                w.Write("Length");
                w.Write(counter);
                w.Write("=");
                if (t.Duration > 0) w.WriteLine(t.Duration); else w.WriteLine(-1);

                counter++;
            }

            w.WriteLine("");

            w.Write("NumberOfEntries=");
            w.WriteLine(tracks.Count);

            w.Write("Version=2");
        }
    }
}
