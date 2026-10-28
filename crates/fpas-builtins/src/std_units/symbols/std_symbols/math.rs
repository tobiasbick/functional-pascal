//! `Std.Math` symbol names and registry group.

std_symbol!(STD_MATH_PI = std_math!("Pi"));
std_symbol!(STD_MATH_SQRT = std_math!("Sqrt"));
std_symbol!(STD_MATH_POW = std_math!("Pow"));
std_symbol!(STD_MATH_FLOOR = std_math!("Floor"));
std_symbol!(STD_MATH_CEIL = std_math!("Ceil"));
std_symbol!(STD_MATH_ROUND = std_math!("Round"));
std_symbol!(STD_MATH_SIN = std_math!("Sin"));
std_symbol!(STD_MATH_COS = std_math!("Cos"));
std_symbol!(STD_MATH_LOG = std_math!("Log"));
std_symbol!(STD_MATH_ABS = std_math!("Abs"));
std_symbol!(STD_MATH_MIN = std_math!("Min"));
std_symbol!(STD_MATH_MAX = std_math!("Max"));
std_symbol!(STD_MATH_TAN = std_math!("Tan"));
std_symbol!(STD_MATH_ARC_SIN = std_math!("ArcSin"));
std_symbol!(STD_MATH_ARC_COS = std_math!("ArcCos"));
std_symbol!(STD_MATH_ARC_TAN = std_math!("ArcTan"));
std_symbol!(STD_MATH_ARC_TAN2 = std_math!("ArcTan2"));
std_symbol!(STD_MATH_EXP = std_math!("Exp"));
std_symbol!(STD_MATH_LOG10 = std_math!("Log10"));
std_symbol!(STD_MATH_LOG2 = std_math!("Log2"));
std_symbol!(STD_MATH_TRUNC = std_math!("Trunc"));
std_symbol!(STD_MATH_FRAC = std_math!("Frac"));
std_symbol!(STD_MATH_SIGN = std_math!("Sign"));
std_symbol!(STD_MATH_CLAMP = std_math!("Clamp"));

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
