//! Built-in intrinsic identifiers (embedded in `Op::Intrinsic`).
//!
//! **Documentation:** `docs/pascal/std/README.md` (from the repository root); each `Std.*` unit page maps API names to these variants.
//! **Maintenance:** When adding or renumbering variants, update that documentation and the affected implementation crates.

pub mod array;
pub mod console;
pub mod conv;
pub mod dict;
pub mod graph;
pub mod math;
pub mod option;
pub mod result;
pub mod str_ops;
pub mod task;
pub mod tui;

pub use array::ArrayIntrinsic;
pub use console::ConsoleIntrinsic;
pub use conv::ConvIntrinsic;
pub use dict::DictIntrinsic;
pub use graph::GraphIntrinsic;
pub use math::MathIntrinsic;
pub use option::OptionIntrinsic;
pub use result::ResultIntrinsic;
pub use str_ops::StrIntrinsic;
pub use task::TaskIntrinsic;
pub use tui::TuiIntrinsic;

/// VM intrinsic opcode payload (`Op::Intrinsic(u16::from(self))`).
///
/// Each variant wraps a domain-specific sub-enum whose discriminant is the stable `u16` wire value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intrinsic {
    /// `Std.Console.*` intrinsics.
    Console(ConsoleIntrinsic),
    /// `Std.Str.*` intrinsics.
    Str(StrIntrinsic),
    /// `Std.Conv.*` intrinsics.
    Conv(ConvIntrinsic),
    /// `Std.Math.*` intrinsics.
    Math(MathIntrinsic),
    /// `Std.Array.*` intrinsics.
    Array(ArrayIntrinsic),
    /// `Std.Dict.*` intrinsics.
    Dict(DictIntrinsic),
    /// `Std.Graph.*` intrinsics.
    Graph(GraphIntrinsic),
    /// `Std.Result.*` intrinsics.
    Result(ResultIntrinsic),
    /// `Std.Option.*` intrinsics.
    Option(OptionIntrinsic),
    /// `Std.Task.*` intrinsics.
    Task(TaskIntrinsic),
    /// `Std.Tui.*` intrinsics.
    Tui(TuiIntrinsic),
}

impl From<Intrinsic> for u16 {
    fn from(intrinsic: Intrinsic) -> Self {
        match intrinsic {
            Intrinsic::Console(x) => x as u16,
            Intrinsic::Str(x) => x as u16,
            Intrinsic::Conv(x) => x as u16,
            Intrinsic::Math(x) => x as u16,
            Intrinsic::Array(x) => x as u16,
            Intrinsic::Dict(x) => x as u16,
            Intrinsic::Graph(x) => x as u16,
            Intrinsic::Result(x) => x as u16,
            Intrinsic::Option(x) => x as u16,
            Intrinsic::Task(x) => x as u16,
            Intrinsic::Tui(x) => x as u16,
        }
    }
}

impl Intrinsic {
    /// Decode a raw `u16` discriminant back to an `Intrinsic` variant.
    ///
    /// Returns `None` for unrecognised values.
    pub fn from_u16(raw: u16) -> Option<Self> {
        if let Ok(x) = ConsoleIntrinsic::try_from(raw) {
            return Some(Self::Console(x));
        }
        if let Ok(x) = StrIntrinsic::try_from(raw) {
            return Some(Self::Str(x));
        }
        if let Ok(x) = ConvIntrinsic::try_from(raw) {
            return Some(Self::Conv(x));
        }
        if let Ok(x) = MathIntrinsic::try_from(raw) {
            return Some(Self::Math(x));
        }
        if let Ok(x) = ArrayIntrinsic::try_from(raw) {
            return Some(Self::Array(x));
        }
        if let Ok(x) = DictIntrinsic::try_from(raw) {
            return Some(Self::Dict(x));
        }
        if let Ok(x) = GraphIntrinsic::try_from(raw) {
            return Some(Self::Graph(x));
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
        if let Ok(x) = TuiIntrinsic::try_from(raw) {
            return Some(Self::Tui(x));
        }
        None
    }
}

#[cfg(test)]
mod tests;
