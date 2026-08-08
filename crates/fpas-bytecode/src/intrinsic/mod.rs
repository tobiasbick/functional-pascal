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
pub mod proc;
pub mod random;
pub mod result;
pub mod str_ops;
pub mod task;
pub mod test;
pub mod time;
pub mod toml;

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
pub use proc::ProcIntrinsic;
pub use random::RandomIntrinsic;
pub use result::ResultIntrinsic;
pub use str_ops::StrIntrinsic;
pub use task::TaskIntrinsic;
pub use test::TestIntrinsic;
pub use time::TimeIntrinsic;
pub use toml::TomlIntrinsic;

/// VM intrinsic opcode payload (`Op::Intrinsic(u16::from(self))`).
///
/// Each variant wraps a domain-specific sub-enum whose discriminant is the stable `u16` wire value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intrinsic {
    Args(ArgsIntrinsic),
    Console(ConsoleIntrinsic),
    Str(StrIntrinsic),
    Conv(ConvIntrinsic),
    Parse(ParseIntrinsic),
    Math(MathIntrinsic),
    Random(RandomIntrinsic),
    Array(ArrayIntrinsic),
    Dict(DictIntrinsic),
    Env(EnvIntrinsic),
    Path(PathIntrinsic),
    Proc(ProcIntrinsic),
    Fs(FsIntrinsic),
    Graph(GraphIntrinsic),
    Json(JsonIntrinsic),
    Result(ResultIntrinsic),
    Option(OptionIntrinsic),
    Task(TaskIntrinsic),
    Time(TimeIntrinsic),
    Toml(TomlIntrinsic),
    Test(TestIntrinsic),
}

macro_rules! intrinsic_wire_ops {
    ($($wrapper:ident($sub:ty)),* $(,)?) => {
        impl From<Intrinsic> for u16 {
            fn from(intrinsic: Intrinsic) -> Self {
                match intrinsic {
                    $(Intrinsic::$wrapper(x) => x as u16,)*
                }
            }
        }

        impl Intrinsic {
            /// Decode a raw `u16` discriminant back to an `Intrinsic` variant.
            ///
            /// Returns `None` for unrecognised values.
            pub fn from_u16(raw: u16) -> Option<Self> {
                $(
                    if let Ok(x) = <$sub>::try_from(raw) {
                        return Some(Self::$wrapper(x));
                    }
                )*
                None
            }

            /// Iterate over every currently assigned intrinsic wire identifier.
            ///
            /// The iterator is derived from the authoritative decoder, so verifier, compiler, and
            /// runtime coverage checks cannot silently diverge from newly assigned IDs.
            pub fn all() -> impl Iterator<Item = Self> {
                (u16::MIN..=u16::MAX).filter_map(Self::from_u16)
            }
        }
    };
}

intrinsic_wire_ops!(
    Args(ArgsIntrinsic),
    Console(ConsoleIntrinsic),
    Str(StrIntrinsic),
    Conv(ConvIntrinsic),
    Parse(ParseIntrinsic),
    Math(MathIntrinsic),
    Random(RandomIntrinsic),
    Array(ArrayIntrinsic),
    Dict(DictIntrinsic),
    Env(EnvIntrinsic),
    Path(PathIntrinsic),
    Proc(ProcIntrinsic),
    Fs(FsIntrinsic),
    Graph(GraphIntrinsic),
    Json(JsonIntrinsic),
    Result(ResultIntrinsic),
    Option(OptionIntrinsic),
    Task(TaskIntrinsic),
    Time(TimeIntrinsic),
    Toml(TomlIntrinsic),
    Test(TestIntrinsic),
);

#[cfg(test)]
mod tests;
