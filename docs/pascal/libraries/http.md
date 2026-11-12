# HTTP requests and bounded UTF-8 text

Use System.Net.Http directly. Keep a configured HttpClient for the caller's
operation or application lifetime and reuse it across requests. See the
[HttpClient lifetime guidelines](https://learn.microsoft.com/en-us/dotnet/fundamentals/networking/http/httpclient-guidelines)
for connection pools, DNS refresh, and factory-managed clients. Each request
and response has its own shorter lifetime.

## Read a response with a size limit

Save this application-local unit as `HttpText.fpas`. GetUtf8 sends a GET request,
accepts successful status codes, and reads strict UTF-8 text. The character limit
counts UTF-16 code units, not wire bytes; a supplementary Unicode character takes
two units. An oversized response fails instead of returning partial text.

```pascal project=http-text
// Application-local HTTP text reading.
unit Recipes.HttpText;

uses
  System as Sys, System.IO as IO, System.Net.Http as Http,
  System.Text as Text, System.Threading as Threading,
  System.Threading.Tasks as Tasks;

// Reads a successful UTF-8 response using a caller-owned reusable client.
public async function GetUtf8(
  Client: Http.HttpClient; Address: Sys.Uri;
  MaximumCharacters: int; Cancellation: Threading.CancellationToken
): Tasks.Task<string>;
requires MaximumCharacters >= 0;
begin
  using const Request: Http.HttpRequestMessage := new Http.HttpRequestMessage(
    Http.HttpMethod.Get,
    Address
  );
  Request.Headers.Accept.ParseAdd('text/plain');
  using const Response: Http.HttpResponseMessage := await Client.SendAsync(
    Request,
    Http.HttpCompletionOption.ResponseHeadersRead, Cancellation
  ).ConfigureAwait(false);
  discard Response.EnsureSuccessStatusCode();
  const Body: IO.Stream := await Response.Content.ReadAsStreamAsync(
    Cancellation
  ).ConfigureAwait(false);
  using const Reader: IO.StreamReader := new IO.StreamReader(
    Body,
    new Text.UTF8Encoding(false, true), false, 4096, true
  );
  const Buffer: array of char := new char[4096];
  const Memory: Sys.Memory<char> := new Sys.Memory<char>(Buffer);
  const Result: Text.StringBuilder := new Text.StringBuilder();
  while true do
    const Count: int := await Reader.ReadAsync(Memory, Cancellation).ConfigureAwait(false);
    if Count = 0 then
      break;
    end if;

    if Count > MaximumCharacters - Result.Length then
      raise new IO.InvalidDataException('HTTP text exceeds the character limit');
    end if;

    discard Result.Append(Buffer, 0, Count);
  end while;

  return Result.ToString();
end function;
end unit;
```

The selected signatures are
[SendAsync(HttpRequestMessage, HttpCompletionOption, CancellationToken)](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient.sendasync?view=net-10.0),
[ReadAsStreamAsync(CancellationToken)](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpcontent.readasstreamasync?view=net-10.0),
and StreamReader.ReadAsync(Memory<char>, CancellationToken). They return
Task<HttpResponseMessage>, Task<Stream>, and ValueTask<int>, respectively.
Memory<char> works across await without requiring a Span signature.

Reader leaves the body stream open; Response owns its content and body stream.
The using declarations dispose Reader, Response, and Request before GetUtf8
completes, including on status failure, oversized text, decoder failure, or
cancellation. The helper leaves the caller's HttpClient available for reuse.

The normal network handler streams the body with
[ResponseHeadersRead](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpcompletionoption?view=net-10.0).
HttpClient.Timeout then covers only the send/header phase, and
MaxResponseContentBufferSize does not limit this streamed body. Pass the same
request/deadline token to both sending and reading, and enforce the body limit
while reading. ReadAsStringAsync buffers the body; use it only with an appropriate
buffering policy. A custom HttpContent/handler can also introduce buffering.

This helper deliberately expects UTF-8; it does not select a decoder from the
response's charset header or a byte-order mark. Invalid UTF-8 throws
DecoderFallbackException. For other encodings, choose and validate the encoding
as part of the application's protocol. Non-success status codes throw
HttpRequestException; cancellation throws OperationCanceledException. Status,
transport, and text failures remain distinct CLR exceptions.

## Own the request deadline

Place this program and the unit in the same executable project. It uses one
client for its operation and a five-second token covering sending and body reads.
In a longer-running application, configure the reusable client at its owner and
link a per-request deadline with the caller's cancellation token.

```pascal project=http-text
async program FetchUtf8;

uses
  System as Sys, System.Console as Console, System.IO as IO,
  System.Net.Http as Http, System.Text as Text, System.Threading as Threading,
  Recipes.HttpText as Fetch;

begin
  using const Client: Http.HttpClient := new Http.HttpClient();
  Client.Timeout := Threading.Timeout.InfiniteTimeSpan;
  using const Deadline: Threading.CancellationTokenSource := new Threading.CancellationTokenSource(
    5000
  );
  try
    const Body: string := await Fetch.GetUtf8(
      Client,
      new Sys.Uri('https://example.com/'),
      16384,
      Deadline.Token
    );
    Console.WriteLine(Body);
  catch Failure: Http.HttpRequestException do
    Console.WriteLine('HTTP request failed');
  catch Failure: Sys.OperationCanceledException do
    Console.WriteLine('HTTP request canceled');
  catch Failure: IO.InvalidDataException do
    Console.WriteLine('HTTP body too large');
  catch Failure: Text.DecoderFallbackException do
    Console.WriteLine('Invalid UTF-8');
  end try;
end;
end program;
```

The executable needs network access for a successful fetch. Its Markdown test
uses a local rejecting proxy to check the HTTP failure branch without public
network access. The linked
[SDK acceptance test](../../../tests/Fpas.IntegrationTests/Build/MsBuildTests.HttpRecipe.cs)
calls the exact unit from C# in Debug/Release. It uses a loopback HTTP endpoint
for a real UTF-8 response and controlled .NET handlers/streams for limits, status
errors, invalid UTF-8, and cancellation during headers and body reads. Those
cases check request/content/stream disposal and client reuse. The test endpoint
is fixture infrastructure, not an FPAS server API.

## See also

- [Library guides](README.md)
- [Cancellation and coordination](../language/concurrency/cancellation-and-coordination.md)
- [Files and streams](files-and-streams.md)
- [Project SDK setup](../../build/standalone-sdk.md)
