//! Enum comparison patterns follow resolved values rather than variant spelling.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/syntax.md`

use super::super::{assert_succeeds, parse_ok};

#[test]
fn is_enum_comparisons_use_shadowing_constant_values() {
    for pattern in ["Red", "(Red)", "((rEd))"] {
        assert_succeeds(&format!(
            "program EnumConstantIs;
type Shade = enum Red = 7; Blue = 42; end enum;
function Matches(Value: Shade): boolean;
begin
  const Red: Shade := Shade.Blue;
  if Value is {pattern} then return true; end if;
  return false;
end function;
function IsRed(Value: Shade): boolean;
begin
  if Value is Shade.Red then return true; end if;
  return false;
end function;
begin
  if not Matches(Shade.Blue) then panic('resolved constant must match blue'); end if;
  if Matches(Shade.Red) then panic('constant spelling must not match red'); end if;
  if not IsRed(Shade.Red) then panic('actual member must match red'); end if;
  if IsRed(Shade.Blue) then panic('actual member must not match blue'); end if;
end."
        ));
    }
}

#[test]
fn exhaustive_nested_cases_use_shadowing_enum_constant_values() {
    for pattern in ["Red", "(Red)", "((rEd))"] {
        assert_succeeds(&format!(
            "program EnumConstantCase;
type Shade = enum Red = 7; Blue = 42; end enum;
function Classify(Value: result of (option of Shade, string)): integer;
begin
  const Red: Shade := Shade.Blue;
  case Value of
    when Ok(Some({pattern})): return 1;
    when Ok(Some(Shade.Red)): return 2;
    when Ok(None): return 3;
    when Error(_): return 4;
  end case;
end function;
begin
  if Classify(Ok(Some(Shade.Blue))) <> 1 then panic('exhaustive blue arm'); end if;
  if Classify(Ok(Some(Shade.Red))) <> 2 then panic('exhaustive red arm'); end if;
  if Classify(Ok(None)) <> 3 then panic('nested none arm'); end if;
  if Classify(Error('stop')) <> 4 then panic('error arm'); end if;
end."
        ));
    }
}

#[test]
fn enum_constant_copies_and_value_paths_preserve_their_resolved_values() {
    assert_succeeds(
        "program EnumValuePaths;
type Shade = enum Red = 7; Blue = 42; end enum;
type Holder = record Red: Shade; end record;
function Copy(Value: Shade): Shade;
begin
  const Red: Shade := Value;
  const Blue: Shade := Red;
  return Blue;
end function;
function Identity(Red: Shade): Shade;
begin
  return Red;
end function;
begin
  const Box: Holder := Holder(Red := Shade.Blue);
  if Copy(Shade.Blue) <> Shade.Blue then panic('copied enum constant'); end if;
  if Copy(Shade.Red) <> Shade.Red then panic('copy must preserve red'); end if;
  if Identity(Shade.Blue) <> Shade.Blue then panic('enum parameter'); end if;
  if Box.Red <> Shade.Blue then panic('enum record field'); end if;
end.",
    );
}

#[test]
fn fieldless_data_enum_values_still_match_variants() {
    assert_succeeds(
        "program FieldlessEnumValues;
type Shape = enum Point; Rect(Width: integer); end enum;
type Holder = record Point: Shape; end record;
function IsPoint(Value: Shape): boolean;
begin
  if Value is Shape.Point then return true; end if;
  return false;
end function;
function Copy(Point: Shape): Shape;
begin
  return Point;
end function;
function Classify(Value: option of Shape): integer;
begin
  case Value of
    when Some(Shape.Point): return 1;
    when Some(Shape.Rect(const Width)): return Width;
    when None: return -1;
  end case;
end function;
begin
  const Box: Holder := Holder(Point := Shape.Rect(8));
  if not IsPoint(Shape.Point) then panic('fieldless variant'); end if;
  if IsPoint(Copy(Shape.Rect(5))) then panic('shadowing data enum parameter'); end if;
  if IsPoint(Box.Point) then panic('data enum record field'); end if;
  if Classify(Some(Shape.Point)) <> 1 then panic('nested fieldless variant'); end if;
  if Classify(Some(Shape.Rect(8))) <> 8 then panic('nested data variant'); end if;
  if Classify(None) <> -1 then panic('none variant'); end if;
end.",
    );
}

#[test]
fn qualified_and_aliased_enum_constants_survive_interface_round_trips() {
    use fpas_unit::interface::{decode_interface, encode_interface};

    let (shades, errors) = fpas_parser::parse_compilation_unit(
        "unit Shades;
public type Shade = enum Red = 7; Blue = 42; end enum;
end unit;",
    );
    assert!(errors.is_empty(), "unit parse errors: {errors:?}");
    let fpas_parser::CompilationUnit::Unit(shades) = shades else {
        panic!("Shades unit expected");
    };
    let mut shades = crate::compile_unit_object(&shades, &[]).expect("compiled Shades");
    shades.interface =
        decode_interface(&encode_interface(&shades.interface).expect("encode Shades"))
            .expect("decode Shades");
    let (palette, errors) = fpas_parser::parse_compilation_unit(
        "unit Palette;
uses Shades as Tone;
public const Red: Tone.Shade := Tone.Shade.Blue;
end unit;",
    );
    assert!(errors.is_empty(), "unit parse errors: {errors:?}");
    let fpas_parser::CompilationUnit::Unit(palette) = palette else {
        panic!("Palette unit expected");
    };
    let mut palette = crate::compile_unit_object(&palette, std::slice::from_ref(&shades.interface))
        .expect("compiled Palette");
    palette.interface =
        decode_interface(&encode_interface(&palette.interface).expect("encode Palette"))
            .expect("decode Palette");
    let interfaces = [shades.interface.clone(), palette.interface.clone()];

    for (import, qualifier) in [("Palette", "Palette"), ("Palette as Colors", "Colors")] {
        for pattern in [format!("{qualifier}.Red"), format!("({qualifier}.Red)")] {
            let program = parse_ok(&format!(
                "program ImportedEnumConstants;
uses Shades as Tone, {import};
function Matches(Value: Tone.Shade): boolean;
begin
  if Value is {pattern} then return true; end if;
  return false;
end function;
function Classify(Value: option of Tone.Shade): integer;
begin
  case Value of
    when Some({pattern}): return 1;
    when Some(Tone.Shade.Red): return 2;
    when None: return 3;
  end case;
end function;
begin
  if not Matches(Tone.Shade.Blue) then panic('qualified constant matches blue'); end if;
  if Matches(Tone.Shade.Red) then panic('qualified constant rejects red'); end if;
  if Classify(Some(Tone.Shade.Blue)) <> 1 then panic('qualified blue arm'); end if;
  if Classify(Some(Tone.Shade.Red)) <> 2 then panic('qualified red arm'); end if;
  if Classify(None) <> 3 then panic('qualified none arm'); end if;
end."
            ));
            let object =
                crate::compile_program_object_with_support(&program, &interfaces, &interfaces)
                    .expect("compiled consumer");
            let executable = fpas_linker::link_objects(
                &[shades.object.clone(), palette.object.clone()],
                &object,
            )
            .expect("linked consumer");
            fpas_vm::Vm::new(executable)
                .run()
                .expect("qualified enum comparisons should execute");
        }
    }
}
