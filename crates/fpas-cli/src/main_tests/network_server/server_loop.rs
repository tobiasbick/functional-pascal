use std::time::Duration;

use super::{exchange, finish_server, start_server, unused_port};

#[test]
fn fpas_http_server_loop_dispatches_multiple_requests() {
    let port = unused_port();
    let (cwd, server) = start_server(
        "http-server-loop",
        format!(
            "program HttpServerLoop;\n\nuses Std.Console as Console; uses Std.Http as Http; uses Std.Net as Net; uses Std.Net.Utf8 as Utf8;\n\nfunction Handle(RequestValue: Http.ServerRequest): Http.ServerResponse;\nbegin\n  mutable var ResponseValue: Http.ServerResponse := Http.ServerResponse.Create(200, 'OK');\n  ResponseValue.Body := Utf8.Encode(RequestValue.Target);\n  return ResponseValue;\nend function;\n\nbegin\n  case Net.Listen('127.0.0.1', {port}) of\n    when Ok(ListenerValue):\n    begin\n      mutable var Options: Http.ServerOptions := Http.ServerOptions.Create();\n      Options.MaxConcurrentRequests := 2;\n      Options.MaxRequests := 2;\n      case Http.Serve(ListenerValue, Options, Handle) of\n        when Ok(_):\n        begin null;\n        end;\n        when Error(Message): panic(Message);\n      end case;\n      case Net.CloseListener(ListenerValue) of\n        when Ok(_): Console.WriteLn('served');\n        when Error(Message): panic(Message);\n      end case;\n    end;\n    when Error(Message): panic(Message);\n  end case;\nend program;\n"
        ),
    );

    let first = std::thread::spawn(move || {
        exchange(port, &[b"GET /first HTTP/1.1\r\nHost: localhost\r\n\r\n"])
    });
    std::thread::sleep(Duration::from_millis(20));
    let second = std::thread::spawn(move || {
        exchange(port, &[b"GET /second HTTP/1.1\r\nHost: localhost\r\n\r\n"])
    });

    let first_response = first.join().expect("first HTTP client must finish");
    let second_response = second.join().expect("second HTTP client must finish");
    let (stdout, _) = finish_server(cwd, server);

    assert_eq!(stdout, "served\n");
    assert_eq!(
        first_response,
        b"HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: 6\r\n\r\n/first"
    );
    assert_eq!(
        second_response,
        b"HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: 7\r\n\r\n/second"
    );
}

#[test]
fn fpas_http_server_loop_isolates_malformed_requests() {
    let port = unused_port();
    let (cwd, server) = start_server(
        "http-server-loop-invalid-request",
        format!(
            "program HttpServerLoopInvalidRequest;\n\nuses Std.Http as Http; uses Std.Net as Net;\n\nfunction Handle(_RequestValue: Http.ServerRequest): Http.ServerResponse;\nbegin\n  return Http.ServerResponse.Create(204, 'No Content');\nend function;\n\nbegin\n  case Net.Listen('127.0.0.1', {port}) of\n    when Ok(ListenerValue):\n    begin\n      mutable var Options: Http.ServerOptions := Http.ServerOptions.Create();\n      Options.MaxConcurrentRequests := 2;\n      Options.MaxRequests := 2;\n      case Http.Serve(ListenerValue, Options, Handle) of\n        when Ok(_):\n        begin null;\n        end;\n        when Error(Message): panic(Message);\n      end case;\n      case Net.CloseListener(ListenerValue) of\n        when Ok(_):\n        begin null;\n        end;\n        when Error(Message): panic(Message);\n      end case;\n    end;\n    when Error(Message): panic(Message);\n  end case;\nend program;\n"
        ),
    );

    let invalid = std::thread::spawn(move || {
        exchange(port, &[b"GET / HTTP/1.1\r\nHost: one\r\nHost: two\r\n\r\n"])
    });
    let valid =
        std::thread::spawn(move || exchange(port, &[b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n"]));

    let invalid_response = invalid.join().expect("invalid HTTP client must finish");
    let valid_response = valid.join().expect("valid HTTP client must finish");
    finish_server(cwd, server);

    assert_eq!(
        invalid_response,
        b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
    );
    assert_eq!(
        valid_response,
        b"HTTP/1.1 204 No Content\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
    );
}

#[test]
fn fpas_http_server_loop_rejects_invalid_options_before_accepting() {
    let port = unused_port();
    let (cwd, server) = start_server(
        "http-server-loop-invalid-options",
        format!(
            "program HttpServerLoopInvalidOptions;\n\nuses Std.Console as Console; uses Std.Http as Http; uses Std.Net as Net; uses Std.Str as Str;\n\nfunction Handle(_RequestValue: Http.ServerRequest): Http.ServerResponse;\nbegin\n  return Http.ServerResponse.Create(204, 'No Content');\nend function;\n\nbegin\n  case Net.Listen('127.0.0.1', {port}) of\n    when Ok(ListenerValue):\n    begin\n      mutable var Options: Http.ServerOptions := Http.ServerOptions.Create();\n      Options.MaxConcurrentRequests := 0;\n      case Http.Serve(ListenerValue, Options, Handle) of\n        when Ok(_): panic('invalid server options were accepted');\n        when Error(Message):\n        begin\n          if not Str.Contains(Message, 'MaxConcurrentRequests') then\n          begin\n            panic(Message);\n          end; end if;\n        end;\n      end case;\n      case Net.CloseListener(ListenerValue) of\n        when Ok(_): Console.WriteLn('rejected');\n        when Error(Message): panic(Message);\n      end case;\n    end;\n    when Error(Message): panic(Message);\n  end case;\nend program;\n"
        ),
    );

    let (stdout, _) = finish_server(cwd, server);

    assert_eq!(stdout, "rejected\n");
}
