//! Built-in intrinsic identifiers embedded in [`crate::Opcode::Intrinsic`] instructions.
//!
//! **Documentation:** `docs/pascal/std/README.md` (from the repository root); each `Std.*` unit page maps API names to these variants.
//! **Maintenance:** When adding or renumbering variants, update that documentation and the affected implementation crates.

macro_rules! documented_intrinsic_enum {
    (
        $(#[$enum_meta:meta])*
        pub enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $value:literal,
            )*
        }
    ) => {
        $(#[$enum_meta])*
        pub enum $name {
            $(
                $(#[$variant_meta])*
                #[doc = concat!("Intrinsic selector `", stringify!($variant), "`.")]
                $variant = $value,
            )*
        }
    };
}

pub mod args;
pub mod array;
pub mod console;
pub mod conv;
pub mod dict;
pub mod env;
mod execution;
pub mod fs;
pub mod http;
pub mod json;
pub mod math;
pub mod net;
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
pub use execution::IntrinsicOwner;
pub use fs::FsIntrinsic;
pub use http::HttpIntrinsic;
pub use json::JsonIntrinsic;
pub use math::MathIntrinsic;
pub use net::NetIntrinsic;
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

/// VM intrinsic instruction payload.
///
/// Each variant wraps a domain-specific sub-enum whose discriminant is the stable `u16` wire value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intrinsic {
    /// Process-argument operation.
    Args(ArgsIntrinsic),
    /// Console operation.
    Console(ConsoleIntrinsic),
    /// String operation.
    Str(StrIntrinsic),
    /// Scalar conversion operation.
    Conv(ConvIntrinsic),
    /// Fallible text-parsing operation.
    Parse(ParseIntrinsic),
    /// Mathematical operation.
    Math(MathIntrinsic),
    /// Hosted TCP networking operation.
    Net(NetIntrinsic),
    /// Internal hosted HTTP operation.
    Http(HttpIntrinsic),
    /// Pseudorandom-number operation.
    Random(RandomIntrinsic),
    /// Array operation.
    Array(ArrayIntrinsic),
    /// Dictionary operation.
    Dict(DictIntrinsic),
    /// Environment-variable operation.
    Env(EnvIntrinsic),
    /// Host-path operation.
    Path(PathIntrinsic),
    /// Host-process operation.
    Proc(ProcIntrinsic),
    /// Filesystem operation.
    Fs(FsIntrinsic),
    /// JSON operation.
    Json(JsonIntrinsic),
    /// Result operation.
    Result(ResultIntrinsic),
    /// Option operation.
    Option(OptionIntrinsic),
    /// Task operation.
    Task(TaskIntrinsic),
    /// Time operation.
    Time(TimeIntrinsic),
    /// TOML operation.
    Toml(TomlIntrinsic),
    /// Test-support operation.
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
    /// Return the canonical FPAS source name used by debugger call binding.
    #[must_use]
    pub fn debugger_name(self) -> String {
        match self {
            Self::Str(StrIntrinsic::Repeat) => "Std.Str.RepeatStr".to_string(),
            Self::Http(HttpIntrinsic::ReserveBodyStreamState) => {
                "Std.Http.Stream.ReserveState".to_string()
            }
            Self::Http(HttpIntrinsic::HasBodyStreamState) => {
                "Std.Http.Stream.HasState".to_string()
            }
            Self::Http(HttpIntrinsic::LoadBodyStreamState) => {
                "Std.Http.Stream.LoadState".to_string()
            }
            Self::Http(HttpIntrinsic::StoreBodyStreamState) => {
                "Std.Http.Stream.StoreState".to_string()
            }
            Self::Http(HttpIntrinsic::ReserveSseDecoderState) => {
                "Std.Http.Sse.ReserveState".to_string()
            }
            Self::Http(HttpIntrinsic::HasSseDecoderState) => {
                "Std.Http.Sse.HasState".to_string()
            }
            Self::Http(HttpIntrinsic::LoadSseDecoderState) => {
                "Std.Http.Sse.LoadState".to_string()
            }
            Self::Http(HttpIntrinsic::StoreSseDecoderState) => {
                "Std.Http.Sse.StoreState".to_string()
            }
            Self::Test(
                TestIntrinsic::AssertEqualsInteger
                | TestIntrinsic::AssertEqualsBoolean
                | TestIntrinsic::AssertEqualsString
                | TestIntrinsic::AssertEqualsReal,
            ) => "Std.Test.AssertEquals".to_string(),
            intrinsic => {
                let debug = format!("{intrinsic:?}");
                let (family, member) = debug
                    .split_once('(')
                    .expect("intrinsic debug representation has family and member");
                format!("Std.{family}.{}", member.trim_end_matches(')'))
            }
        }
    }

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
    Net(NetIntrinsic),
    Http(HttpIntrinsic),
    Random(RandomIntrinsic),
    Array(ArrayIntrinsic),
    Dict(DictIntrinsic),
    Env(EnvIntrinsic),
    Path(PathIntrinsic),
    Proc(ProcIntrinsic),
    Fs(FsIntrinsic),
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
