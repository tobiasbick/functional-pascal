//! Character-code literals retain compile-time identity and runtime scalar semantics.
//! See `docs/pascal/language/basics/primitive-types.md`.

use super::super::assert_succeeds;

#[test]
fn unicode_character_codes_match_runtime_factories_and_constant_case_labels() {
    assert_succeeds(
        "program UnicodeLiterals;
        const Euro: string := #8_364;
        const Label: string := 'A'#8364#128512'Z';
        begin
          const Characters := [#0, #255, #256, #8364, #55295, #57344, #128512, #1114111];
          const Codes := [0, 255, 256, 8364, 55295, 57344, 128512, 1114111];
          for Index: integer := 0 to Codes.Length() - 1 do
            if (Characters[Index] <> string.Chr(Codes[Index])) or
               (Characters[Index].Length() <> 1) or
               (Characters[Index].Ord() <> Codes[Index]) then
              panic('Character-code scalar mismatch');
            end if;
          end for;
          if (Euro <> '€') or (Label <> 'A€😀Z') or (Label.Length() <> 4) or
             (Label[1] <> Euro) or (Label[2] <> '😀') then
            panic('Adjacent character-code values or scalar indexing');
          end if;
          case 'A€😀Z' of
            when Label: null;
            else panic('Character-code constant label mismatch');
          end case;
        end.",
    );
}
