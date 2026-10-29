//! Terminal authentication shares the existing foreground process implementation.
use super::{
    acp::{AcpLaunch, fault},
    options::{Options, Request},
    registry::Processes,
};
use crate::execution::InteractiveProcessTerminal;
use fpas_acp::{connection::Fault, protocol::v1::AuthMethodTerminal};
use std::time::Duration;

pub(in crate::execution::vm::hosted) struct Authentication {
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}
impl From<AuthMethodTerminal> for Authentication {
    fn from(method: AuthMethodTerminal) -> Self {
        Self {
            args: method.args,
            env: method.env.into_iter().collect(),
        }
    }
}
impl From<fpas_acp::protocol::v2::AuthMethodTerminal> for Authentication {
    fn from(method: fpas_acp::protocol::v2::AuthMethodTerminal) -> Self {
        Self {
            args: method.args,
            env: method
                .env
                .into_iter()
                .map(|item| (item.name, item.value))
                .collect(),
        }
    }
}

fn merge(base: &AcpLaunch, method: &Authentication) -> AcpLaunch {
    let mut launch = base.clone();
    launch.arguments.extend(method.args.iter().cloned());
    // Use the OS's environment-key comparison, including case folding on Windows.
    let mut environment = std::process::Command::new("acp-auth-configuration");
    for (name, value) in &launch.environment {
        if let Some(value) = value {
            environment.env(name, value);
        } else {
            environment.env_remove(name);
        }
    }
    for (name, value) in &method.env {
        environment.env(name, value);
    }
    launch.environment = environment
        .get_envs()
        .map(|(name, value)| {
            (
                name.to_string_lossy().into_owned(),
                value.map(|value| value.to_string_lossy().into_owned()),
            )
        })
        .collect();
    launch
}

fn request(launch: &AcpLaunch, timeout: Option<Duration>) -> Request<'_> {
    Request {
        command: &launch.command,
        arguments: launch.arguments.iter().map(String::as_str).collect(),
        options: Options {
            directory: launch.directory.as_deref(),
            environment: launch
                .environment
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_deref()))
                .collect(),
            timeout,
            ..Options::default()
        },
        capture: false,
        inherit_input: true,
    }
}

#[cfg(test)]
pub(in crate::execution::vm::hosted) fn authenticate(
    base: &AcpLaunch,
    method: &Authentication,
    timeout: Option<Duration>,
    terminal: Option<&InteractiveProcessTerminal>,
    processes: &Processes,
    cancelled: impl FnMut() -> bool,
) -> Result<bool, Fault> {
    authenticate_at(
        base,
        method,
        timeout,
        terminal,
        processes,
        cancelled,
        std::time::Instant::now(),
    )
}
pub(in crate::execution::vm::hosted) fn authenticate_at(
    base: &AcpLaunch,
    method: &Authentication,
    timeout: Option<Duration>,
    terminal: Option<&InteractiveProcessTerminal>,
    processes: &Processes,
    mut cancelled: impl FnMut() -> bool,
    started: std::time::Instant,
) -> Result<bool, Fault> {
    request(base, timeout).validate().map_err(fault)?;
    Request {
        command: &base.command,
        arguments: method.args.iter().map(String::as_str).collect(),
        options: Options {
            environment: method
                .env
                .iter()
                .map(|(name, value)| (name.as_str(), Some(value.as_str())))
                .collect(),
            ..Options::default()
        },
        capture: false,
        inherit_input: true,
    }
    .validate()
    .map_err(fault)?;
    let launch = merge(base, method);
    let request = request(&launch, timeout);
    request.validate().map_err(fault)?;
    if cancelled() {
        return Err(Fault::Cancelled);
    }
    if timeout.is_some_and(|timeout| started.elapsed() >= timeout) {
        return Err(Fault::Timeout);
    }
    if terminal.is_none() {
        return Err(Fault::UnsupportedCapability("InteractiveTerminal"));
    }
    processes
        .run_interactive_at(&request, terminal, cancelled, started)
        .map(|status| status.code() == Some(0))
        .map_err(fault)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authentication_merges_once_and_never_launches_without_a_host_terminal() {
        let base = AcpLaunch {
            command: "must-not-launch-acp-auth".into(),
            arguments: vec!["base".into()],
            directory: Some("/workspace".into()),
            environment: vec![
                ("ACP_BASE".into(), Some("kept".into())),
                ("ACP_OVERRIDE".into(), None),
            ],
        };
        let method = AuthMethodTerminal::new("login", "Login")
            .args(vec!["auth".into(), "login".into()])
            .env([(String::from("ACP_OVERRIDE"), String::from("changed"))].into());
        let method = Authentication::from(method);
        let merged = merge(&base, &method);
        assert_eq!(merged.arguments, ["base", "auth", "login"]);
        assert_eq!(merged.directory.as_deref(), Some("/workspace"));
        assert!(
            merged
                .environment
                .contains(&("ACP_BASE".into(), Some("kept".into())))
        );
        assert!(
            merged
                .environment
                .contains(&("ACP_OVERRIDE".into(), Some("changed".into())))
        );
        let processes = Processes::new();
        assert_eq!(
            authenticate_at(
                &base,
                &method,
                Some(Duration::from_millis(1)),
                Some(&InteractiveProcessTerminal::new(|| panic!(
                    "expired admission must precede provider access"
                ))),
                &processes,
                || false,
                std::time::Instant::now() - Duration::from_secs(1)
            ),
            Err(Fault::Timeout)
        );
        let duplicate = Authentication::from(
            fpas_acp::protocol::v2::AuthMethodTerminal::new("login", "Login").env(vec![
                fpas_acp::protocol::v2::EnvVariable::new("ACP_DUP", "first"),
                fpas_acp::protocol::v2::EnvVariable::new("ACP_DUP", "second"),
            ]),
        );
        assert_eq!(
            authenticate(&base, &duplicate, None, None, &processes, || false),
            Err(Fault::InvalidArgument("Environment"))
        );
        assert_eq!(
            authenticate(&base, &method, None, None, &processes, || false),
            Err(Fault::UnsupportedCapability("InteractiveTerminal"))
        );
        assert_eq!(
            authenticate(&base, &method, None, None, &processes, || true),
            Err(Fault::Cancelled)
        );
        assert_eq!(
            authenticate(
                &base,
                &method,
                Some(Duration::ZERO),
                None,
                &processes,
                || false
            ),
            Err(Fault::Timeout)
        );
    }
}
