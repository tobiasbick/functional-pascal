using System.IO;
using System.Collections.Generic;
using System;
using Commons;

namespace ATL.Playlist.IO
{
    /// <summary>
    /// Foobar2000 EXPERIMENTAL playlist reader
    /// Since the format is not open and can be subject to change by
    /// fb2k developers at any time, this reader is experimental : it is not guaranteed 
    /// to work with all versions of FPL files
    /// </summary>
    public class FPLIO : PlaylistIO
    {
        private static readonly byte[] FILE_IDENTIFIER = Utils.Latin1Encoding.GetBytes("file://");
        private static readonly byte[] HTTP_IDENTIFIER = Utils.Latin1Encoding.GetBytes("http://");
        private static readonly byte[] HTTPS_IDENTIFIER = Utils.Latin1Encoding.GetBytes("https://");

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="filePath">Path of playlist file to load</param>
        /// <param name="readTrackMetadata">Whether to open referenced media.</param>
        public FPLIO(string filePath, bool readTrackMetadata = true) : base(filePath, readTrackMetadata: readTrackMetadata)
        {
        }

        /// <inheritdoc/>
        protected override void load(FileStream fs, IList<FileLocation> locations, IList<Track> tracks)
        {
            var references = new SortedDictionary<long, string>();
            foreach (var identifier in new[] { FILE_IDENTIFIER, HTTP_IDENTIFIER, HTTPS_IDENTIFIER })
            {
                fs.Seek(0, SeekOrigin.Begin);
                while (StreamUtils.FindSequence(fs, identifier))
                {
                    var start = fs.Position - identifier.Length;
                    fs.Seek(start, SeekOrigin.Begin);
                    references[start] = StreamUtils.ReadNullTerminatedString(fs, UTF8_NO_BOM);
                }
            }
            foreach (var reference in references.Values)
            {
                var location = decodeLocation(reference);
                locations.Add(location);
                tracks.Add(new Track(location.Path, readTrackMetadata));
            }
        }

        /// <inheritdoc/>
        protected override void save(FileStream fs, IList<Track> tracks)
        {
            throw new NotImplementedException("FPL writing not implemented");
        }
    }
}
