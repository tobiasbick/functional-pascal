//! Source-token preprocessor for `{$...}` compiler directives.
//!
//! The preprocessor runs after lexing and before parsing.  It consumes
//! [`Token::Directive`] tokens, evaluates conditional compilation blocks, and
//! returns a filtered token stream ready for the parser.
//!
//! # Supported directives
//!
//! | Directive | Effect |
//! |-----------|--------|
//! | `{$DEFINE name}` | Adds `name` to the active symbol set. |
//! | `{$UNDEF name}` | Removes `name` from the active symbol set. |
//! | `{$IFDEF name}` | Starts a block emitted only when `name` is defined. |
//! | `{$IFNDEF name}` | Starts a block emitted only when `name` is **not** defined. |
//! | `{$ELSE}` | Toggles the current conditional block. |
//! | `{$ENDIF}` | Closes the current conditional block. |
//! | `{$I file}` / `{$INCLUDE file}` | Emits an error (file I/O not available here). |
//! | everything else | Emits a warning diagnostic and is ignored. |
//!
//! **Documentation:** `docs/pascal/12-compiler-directives.md`

mod directive;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub use directive::{DirectiveKind, parse_directive_content};

use crate::{LexError, Span, SpannedToken, Token, error::{lex_error, lex_warning}};
use fpas_diagnostics::codes::{
    LEX_DIRECTIVE_ELSE_WITHOUT_IFDEF, LEX_DIRECTIVE_ENDIF_WITHOUT_IFDEF,
    LEX_DIRECTIVE_INCLUDE_UNSUPPORTED, LEX_DIRECTIVE_UNCLOSED_IFDEF, LEX_DIRECTIVE_UNKNOWN,
};

/// A set of defined conditional compilation symbols.
///
/// Symbol names are stored upper-cased so comparison is always case-insensitive.
#[derive(Debug, Default, Clone)]
pub struct DefineSet(HashSet<String>);

impl DefineSet {
    /// Creates an empty symbol set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a symbol set pre-populated from an iterator of names.
    ///
    /// Names are normalised to upper-case automatically.
    pub fn from_iter(iter: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self(iter.into_iter().map(|s| s.into().to_uppercase()).collect())
    }

    /// Returns `true` if `name` (case-insensitive) is defined.
    #[must_use]
    pub fn is_defined(&self, name: &str) -> bool {
        self.0.contains(&name.to_uppercase())
    }

    /// Adds `name` (normalised to upper-case) to the set.
    pub fn define(&mut self, name: impl Into<String>) {
        self.0.insert(name.into().to_uppercase());
    }

    /// Removes `name` (case-insensitive) from the set.
    pub fn undef(&mut self, name: &str) {
        self.0.remove(&name.to_uppercase());
    }
}

/// Processes a raw token stream produced by the lexer and evaluates all
/// `{$...}` compiler directives.
///
/// Tokens inside inactive conditional branches are removed.  Directive tokens
/// themselves are always removed.
///
/// # Arguments
///
/// - `tokens` — The full token stream from [`crate::lex`], including any
///   [`Token::Directive`] tokens.
/// - `defines` — Initial set of defined symbols.  [`DEFINE`][DirectiveKind::Define]
///   and [`UNDEF`][DirectiveKind::Undef] directives mutate a *local copy* of this
///   set for the duration of this file; the caller's set is not modified.
///
/// # Returns
///
/// A pair `(filtered_tokens, errors)`.  Errors include mismatched
/// `{$ELSE}`/`{$ENDIF}`, unclosed blocks, unsupported `{$INCLUDE}`, and
/// unknown directive names.
#[must_use]
pub fn preprocess(
    tokens: Vec<SpannedToken>,
    defines: &DefineSet,
) -> (Vec<SpannedToken>, Vec<LexError>) {
    let state = Preprocessor::new(defines.clone());
    state.run(tokens)
}

/// Processes a token stream in project mode, resolving active `{$I ...}` /
/// `{$INCLUDE ...}` directives relative to `current_file`.
///
/// Included files are lexed and preprocessed recursively as if their source
/// text appeared inline at the include site.
///
/// **Documentation:** `docs/pascal/12-compiler-directives.md`
pub fn preprocess_in_project(
    tokens: Vec<SpannedToken>,
    defines: &DefineSet,
    current_file: &Path,
) -> Result<(Vec<SpannedToken>, Vec<LexError>), String> {
    let state = Preprocessor::new(defines.clone());
    state.run_in_project(tokens, current_file)
}

// ── Internal state ────────────────────────────────────────────────────────────

/// One frame on the conditional compilation stack.
///
/// - `active` — whether we are currently emitting tokens in this frame.
/// - `parent_active` — whether the outer context was emitting when this
///   `{$IFDEF}` / `{$IFNDEF}` was encountered.  Needed to compute the correct
///   `{$ELSE}` state without storing the original condition separately.
/// - `seen_else` — guard against a second `{$ELSE}` inside one block.
struct IfFrame {
    active: bool,
    parent_active: bool,
    seen_else: bool,
    /// Span of the opening `{$IFDEF}` / `{$IFNDEF}` for error reporting.
    open_span: Span,
}

struct Preprocessor {
    defines: DefineSet,
    stack: Vec<IfFrame>,
    out: Vec<SpannedToken>,
    errors: Vec<LexError>,
}

impl Preprocessor {
    fn new(defines: DefineSet) -> Self {
        Self {
            defines,
            stack: Vec::new(),
            out: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Returns `true` when the current nested context is fully active (all
    /// enclosing frames are in their emitting branch).
    fn is_emitting(&self) -> bool {
        self.stack.iter().all(|f| f.active)
    }

    fn run(mut self, tokens: Vec<SpannedToken>) -> (Vec<SpannedToken>, Vec<LexError>) {
        self.process_tokens(tokens, None, true)
            .expect("single-file preprocessing must not perform file I/O");

        self.finish()
    }

    fn run_in_project(
        mut self,
        tokens: Vec<SpannedToken>,
        current_file: &Path,
    ) -> Result<(Vec<SpannedToken>, Vec<LexError>), String> {
        self.process_tokens(tokens, Some(current_file), true)?;
        Ok(self.finish())
    }

    fn finish(mut self) -> (Vec<SpannedToken>, Vec<LexError>) {

        // Report any unclosed IFDEF/IFNDEF blocks.
        for frame in &self.stack {
            self.errors.push(lex_error(
                LEX_DIRECTIVE_UNCLOSED_IFDEF,
                "Unclosed `{$IFDEF}` or `{$IFNDEF}` block",
                "Add a matching `{$ENDIF}` to close this conditional block.",
                frame.open_span,
            ));
        }

        (self.out, self.errors)
    }

    fn process_tokens(
        &mut self,
        tokens: Vec<SpannedToken>,
        current_file: Option<&Path>,
        keep_eof: bool,
    ) -> Result<(), String> {
        for st in tokens {
            match &st.token {
                Token::Directive(raw) => {
                    let span = st.span;
                    let kind = parse_directive_content(raw);
                    self.handle_directive(kind, span, current_file)?;
                }
                Token::Eof if !keep_eof => {}
                _ => {
                    if self.is_emitting() {
                        self.out.push(st);
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_directive(
        &mut self,
        kind: DirectiveKind,
        span: Span,
        current_file: Option<&Path>,
    ) -> Result<(), String> {
        match kind {
            DirectiveKind::IfDef(name) => {
                let parent = self.is_emitting();
                let cond = parent && self.defines.is_defined(&name);
                self.stack.push(IfFrame {
                    active: cond,
                    parent_active: parent,
                    seen_else: false,
                    open_span: span,
                });
            }
            DirectiveKind::IfNDef(name) => {
                let parent = self.is_emitting();
                let cond = parent && !self.defines.is_defined(&name);
                self.stack.push(IfFrame {
                    active: cond,
                    parent_active: parent,
                    seen_else: false,
                    open_span: span,
                });
            }
            DirectiveKind::Else => match self.stack.last_mut() {
                None => {
                    self.errors.push(lex_error(
                        LEX_DIRECTIVE_ELSE_WITHOUT_IFDEF,
                        "`{$ELSE}` without a matching `{$IFDEF}` or `{$IFNDEF}`",
                        "Remove this `{$ELSE}` or add an opening `{$IFDEF name}` before it.",
                        span,
                    ));
                }
                Some(frame) if frame.seen_else => {
                    self.errors.push(lex_error(
                        LEX_DIRECTIVE_ELSE_WITHOUT_IFDEF,
                        "Duplicate `{$ELSE}` inside a single conditional block",
                        "Each `{$IFDEF}` / `{$IFNDEF}` may have at most one `{$ELSE}`.",
                        span,
                    ));
                }
                Some(frame) => {
                    frame.seen_else = true;
                    // Flip: active becomes true only if the parent was emitting AND
                    // the if-branch was inactive (i.e. the condition was false).
                    frame.active = !frame.active && frame.parent_active;
                }
            },
            DirectiveKind::EndIf => {
                if self.stack.pop().is_none() {
                    self.errors.push(lex_error(
                        LEX_DIRECTIVE_ENDIF_WITHOUT_IFDEF,
                        "`{$ENDIF}` without a matching `{$IFDEF}` or `{$IFNDEF}`",
                        "Remove this `{$ENDIF}` or add an opening `{$IFDEF name}` before it.",
                        span,
                    ));
                }
            }
            DirectiveKind::Define(name) => {
                if self.is_emitting() {
                    self.defines.define(name);
                }
            }
            DirectiveKind::Undef(name) => {
                if self.is_emitting() {
                    self.defines.undef(&name);
                }
            }
            DirectiveKind::Include(filename) => {
                if self.is_emitting() {
                    match current_file {
                        Some(path) => self.process_include(path, &filename)?,
                        None => {
                            self.errors.push(lex_error(
                                LEX_DIRECTIVE_INCLUDE_UNSUPPORTED,
                                &format!("`{{$INCLUDE {filename}}}` is not supported in single-file mode"),
                                "File inclusion is only available inside a multi-file project. \
                                 Use `fpas build` with a `.fpasprj` project file.",
                                span,
                            ));
                        }
                    }
                }
            }
            DirectiveKind::Unknown(raw) => {
                if self.is_emitting() {
                    self.errors.push(lex_warning(
                        LEX_DIRECTIVE_UNKNOWN,
                        &format!("Unknown compiler directive `{{${raw}}}`"),
                        "Supported directives: DEFINE, UNDEF, IFDEF, IFNDEF, ELSE, ENDIF, \
                         INCLUDE (project mode only).  Compiler switches such as `{$R+}` are \
                         accepted but have no effect.",
                        span,
                    ));
                }
            }
        }

        Ok(())
    }

    fn process_include(&mut self, current_file: &Path, filename: &str) -> Result<(), String> {
        let include_path = resolve_include_path(current_file, filename)?;
        let source = fs::read_to_string(&include_path).map_err(|error| {
            format!(
                "Error reading included file `{}` referenced from `{}`: {error}",
                include_path.to_string_lossy(),
                current_file.to_string_lossy()
            )
        })?;
        let (tokens, lex_errors) = crate::lex(&source);
        self.errors.extend(lex_errors);
        self.process_tokens(tokens, Some(&include_path), false)
    }
}

fn resolve_include_path(current_file: &Path, filename: &str) -> Result<PathBuf, String> {
    let trimmed = filename.trim();
    if trimmed.is_empty() {
        return Err(format!(
            "Include directive in `{}` is missing a file name.",
            current_file.to_string_lossy()
        ));
    }

    let raw_path = PathBuf::from(trimmed);
    if raw_path.is_absolute() {
        return Ok(raw_path);
    }

    let base_dir = current_file.parent().ok_or_else(|| {
        format!(
            "Cannot resolve include path `{trimmed}` relative to `{}`.",
            current_file.to_string_lossy()
        )
    })?;
    Ok(base_dir.join(raw_path))
}
