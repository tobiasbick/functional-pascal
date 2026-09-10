"""Regression tests for Functional Pascal training-data content rules."""

from __future__ import annotations

import unittest

from dataset_rules import forbidden_reason


class DatasetRulesTests(unittest.TestCase):
    """Keep obsolete APIs and foreign syntax out of positive examples."""

    def test_rejects_legacy_standard_library_names(self) -> None:
        """Old collection and result unit names must not return."""
        for content in ("uses Std.Array;", "Std.Dict.Get(D, 'x')", "Std.Option.Unwrap(V)"):
            with self.subTest(content=content):
                self.assertIsNotNone(forbidden_reason(content))

    def test_rejects_identifiers_that_became_reserved(self) -> None:
        """Renamed APIs must use identifiers that are not FPAS keywords."""
        for content in (
            "Std.Console.Read()",
            "Std.Net.Write(Connection, Bytes)",
            "TomlValue.Array(Values)",
            "var E: Event := ReadEvent();",
        ):
            with self.subTest(content=content):
                self.assertIsNotNone(forbidden_reason(content))

    def test_rejects_foreign_match_with_syntax(self) -> None:
        """Generic ML-style match syntax is not valid FPAS."""
        self.assertIsNotNone(forbidden_reason("match MaybeValue with\n  | Some(X) => X"))

    def test_accepts_current_names_and_case_syntax(self) -> None:
        """The current API names and FPAS case form remain valid positive data."""
        content = (
            "uses Std.Arrays, Std.Dictionaries, Std.Options, Std.Results;\n"
            "var E: ConsoleEvent := ReadEvent();\n"
            "case MaybeValue of Some(X): WriteLn(X) else WriteLn('none') end"
        )
        self.assertIsNone(forbidden_reason(content))


if __name__ == "__main__":
    unittest.main()
