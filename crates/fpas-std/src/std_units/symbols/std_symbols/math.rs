//! `Std.Math` symbol names and registry group.

pub const STD_MATH_PI: &str = std_math!("Pi");
pub const STD_MATH_SQRT: &str = std_math!("Sqrt");
pub const STD_MATH_POW: &str = std_math!("Pow");
pub const STD_MATH_FLOOR: &str = std_math!("Floor");
pub const STD_MATH_CEIL: &str = std_math!("Ceil");
pub const STD_MATH_ROUND: &str = std_math!("Round");
pub const STD_MATH_SIN: &str = std_math!("Sin");
pub const STD_MATH_COS: &str = std_math!("Cos");
pub const STD_MATH_LOG: &str = std_math!("Log");
pub const STD_MATH_ABS: &str = std_math!("Abs");
pub const STD_MATH_MIN: &str = std_math!("Min");
pub const STD_MATH_MAX: &str = std_math!("Max");
pub const STD_MATH_TAN: &str = std_math!("Tan");
pub const STD_MATH_ARC_SIN: &str = std_math!("ArcSin");
pub const STD_MATH_ARC_COS: &str = std_math!("ArcCos");
pub const STD_MATH_ARC_TAN: &str = std_math!("ArcTan");
pub const STD_MATH_ARC_TAN2: &str = std_math!("ArcTan2");
pub const STD_MATH_EXP: &str = std_math!("Exp");
pub const STD_MATH_LOG10: &str = std_math!("Log10");
pub const STD_MATH_LOG2: &str = std_math!("Log2");
pub const STD_MATH_TRUNC: &str = std_math!("Trunc");
pub const STD_MATH_FRAC: &str = std_math!("Frac");
pub const STD_MATH_SIGN: &str = std_math!("Sign");
pub const STD_MATH_CLAMP: &str = std_math!("Clamp");

pub(in crate::std_units) const STD_MATH_SYMBOLS: &[&str] = &[
    STD_MATH_PI,
    STD_MATH_SQRT,
    STD_MATH_POW,
    STD_MATH_FLOOR,
    STD_MATH_CEIL,
    STD_MATH_ROUND,
    STD_MATH_SIN,
    STD_MATH_COS,
    STD_MATH_LOG,
    STD_MATH_ABS,
    STD_MATH_MIN,
    STD_MATH_MAX,
    STD_MATH_TAN,
    STD_MATH_ARC_SIN,
    STD_MATH_ARC_COS,
    STD_MATH_ARC_TAN,
    STD_MATH_ARC_TAN2,
    STD_MATH_EXP,
    STD_MATH_LOG10,
    STD_MATH_LOG2,
    STD_MATH_TRUNC,
    STD_MATH_FRAC,
    STD_MATH_SIGN,
    STD_MATH_CLAMP,
];
