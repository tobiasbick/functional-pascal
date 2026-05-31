//! Built-in intrinsic identifiers (embedded in `Op::Intrinsic`).
//!
//! **Documentation:** `docs/pascal/std/README.md` (from the repository root); each `Std.*` unit page maps API names to these variants.
//! **Maintenance:** When adding or renumbering variants, update that documentation and the affected implementation crates.

pub mod args;
pub mod array;
pub mod console;
pub mod conv;
pub mod dict;
pub mod env;
pub mod fs;
pub mod graph;
pub mod json;
pub mod math;
pub mod option;
pub mod parse;
pub mod path;
pub mod random;
pub mod result;
pub mod str_ops;
pub mod task;
pub mod time;
pub mod tui;

pub use args::ArgsIntrinsic;
pub use array::ArrayIntrinsic;
pub use console::ConsoleIntrinsic;
pub use conv::ConvIntrinsic;
pub use dict::DictIntrinsic;
pub use env::EnvIntrinsic;
pub use fs::FsIntrinsic;
pub use graph::GraphIntrinsic;
pub use json::JsonIntrinsic;
pub use math::MathIntrinsic;
pub use option::OptionIntrinsic;
pub use parse::ParseIntrinsic;
pub use path::PathIntrinsic;
pub use random::RandomIntrinsic;
pub use result::ResultIntrinsic;
pub use str_ops::StrIntrinsic;
pub use task::TaskIntrinsic;
pub use time::TimeIntrinsic;
pub use tui::TuiIntrinsic;

/// VM intrinsic opcode payload (`Op::Intrinsic(u16::from(self))`).
///
/// Each variant wraps a domain-specific sub-enum whose discriminant is the stable `u16` wire value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intrinsic {
    /// `Std.Args.*` intrinsics.
    Args(ArgsIntrinsic),
    /// `Std.Console.*` intrinsics.
    Console(ConsoleIntrinsic),
    /// `Std.Str.*` intrinsics.
    Str(StrIntrinsic),
    /// `Std.Conv.*` intrinsics.
    Conv(ConvIntrinsic),
    /// `Std.Parse.*` intrinsics.
    Parse(ParseIntrinsic),
    /// `Std.Math.*` intrinsics.
    Math(MathIntrinsic),
    /// `Std.Random.*` intrinsics.
    Random(RandomIntrinsic),
    /// `Std.Array.*` intrinsics.
    Array(ArrayIntrinsic),
    /// `Std.Dict.*` intrinsics.
    Dict(DictIntrinsic),
    /// `Std.Env.*` intrinsics.
    Env(EnvIntrinsic),
    /// `Std.Path.*` intrinsics.
    Path(PathIntrinsic),
    /// `Std.Fs.*` intrinsics.
    Fs(FsIntrinsic),
    /// `Std.Graph.*` intrinsics.
    Graph(GraphIntrinsic),
    /// `Std.Json.*` intrinsics.
    Json(JsonIntrinsic),
    /// `Std.Result.*` intrinsics.
    Result(ResultIntrinsic),
    /// `Std.Option.*` intrinsics.
    Option(OptionIntrinsic),
    /// `Std.Task.*` intrinsics.
    Task(TaskIntrinsic),
    /// `Std.Time.*` intrinsics.
    Time(TimeIntrinsic),
    /// `Std.Tui.*` intrinsics.
    Tui(TuiIntrinsic),
}

impl From<Intrinsic> for u16 {
    fn from(intrinsic: Intrinsic) -> Self {
        match intrinsic {
            Intrinsic::Args(x) => x as u16,
            Intrinsic::Console(x) => x as u16,
            Intrinsic::Str(x) => x as u16,
            Intrinsic::Conv(x) => x as u16,
            Intrinsic::Parse(x) => x as u16,
            Intrinsic::Math(x) => x as u16,
            Intrinsic::Random(x) => x as u16,
            Intrinsic::Array(x) => x as u16,
            Intrinsic::Dict(x) => x as u16,
            Intrinsic::Env(x) => x as u16,
            Intrinsic::Path(x) => x as u16,
            Intrinsic::Fs(x) => x as u16,
            Intrinsic::Graph(x) => x as u16,
            Intrinsic::Json(x) => x as u16,
            Intrinsic::Result(x) => x as u16,
            Intrinsic::Option(x) => x as u16,
            Intrinsic::Task(x) => x as u16,
            Intrinsic::Time(x) => x as u16,
            Intrinsic::Tui(x) => x as u16,
        }
    }
}

impl Intrinsic {
    /// Decode a raw `u16` discriminant back to an `Intrinsic` variant.
    ///
    /// Returns `None` for unrecognised values.
    pub fn from_u16(raw: u16) -> Option<Self> {
        if let Ok(x) = ArgsIntrinsic::try_from(raw) {
            return Some(Self::Args(x));
        }
        if let Ok(x) = ConsoleIntrinsic::try_from(raw) {
            return Some(Self::Console(x));
        }
        if let Ok(x) = StrIntrinsic::try_from(raw) {
            return Some(Self::Str(x));
        }
        if let Ok(x) = ConvIntrinsic::try_from(raw) {
            return Some(Self::Conv(x));
        }
        if let Ok(x) = ParseIntrinsic::try_from(raw) {
            return Some(Self::Parse(x));
        }
        if let Ok(x) = MathIntrinsic::try_from(raw) {
            return Some(Self::Math(x));
        }
        if let Ok(x) = RandomIntrinsic::try_from(raw) {
            return Some(Self::Random(x));
        }
        if let Ok(x) = ArrayIntrinsic::try_from(raw) {
            return Some(Self::Array(x));
        }
        if let Ok(x) = DictIntrinsic::try_from(raw) {
            return Some(Self::Dict(x));
        }
        if let Ok(x) = EnvIntrinsic::try_from(raw) {
            return Some(Self::Env(x));
        }
        if let Ok(x) = PathIntrinsic::try_from(raw) {
            return Some(Self::Path(x));
        }
        if let Ok(x) = FsIntrinsic::try_from(raw) {
            return Some(Self::Fs(x));
        }
        if let Ok(x) = GraphIntrinsic::try_from(raw) {
            return Some(Self::Graph(x));
        }
        if let Ok(x) = JsonIntrinsic::try_from(raw) {
            return Some(Self::Json(x));
        }
        if let Ok(x) = ResultIntrinsic::try_from(raw) {
            return Some(Self::Result(x));
        }
        if let Ok(x) = OptionIntrinsic::try_from(raw) {
            return Some(Self::Option(x));
        }
        if let Ok(x) = TaskIntrinsic::try_from(raw) {
            return Some(Self::Task(x));
        }
        if let Ok(x) = TimeIntrinsic::try_from(raw) {
            return Some(Self::Time(x));
        }
        if let Ok(x) = TuiIntrinsic::try_from(raw) {
            return Some(Self::Tui(x));
        }
        None
    }
}

#[cfg(test)]
mod tests;
