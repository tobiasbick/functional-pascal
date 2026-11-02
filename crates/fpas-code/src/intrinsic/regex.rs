//! Compiled Unicode regular-expression operations.
use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Operations owned by `Std.Text.Regex`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum RegexIntrinsic {
    DefaultOptions = 870,
    Compile = 871,
    IsMatch = 872,
    Find = 873,
    FindAll = 874,
    Replace = 875,
    Split = 876,
    Escape = 877,
    GroupNames = 878,
}
}
