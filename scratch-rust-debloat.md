# Rust debloat — Arbeitsstand

Ausgangspunkt: Audit vom 31.10.2026. Änderungen bleiben zunächst uncommitted.
Ein Punkt wird erst nach Umsetzung und passender Prüfung als fertig markiert.
Laufzeitgewinne werden ohne Vergleichsmessung nicht behauptet.

## Checkliste

- [x] 1. ACP-JSON: geprüfte Rohwerte werden als geliehene Teilbäume weitergereicht und erst bei benötigtem Besitz kopiert. Zahlen, unbekannte Felder, Duplikatschlüssel und Tiefengrenzen bleiben geschützt; Codec-/Binding-Tests erfolgreich.
- [x] 2. Compiler-IR: zweite vollständige Validierung vor der Optimierung entfernt. Lowering bleibt die Prüfgrenze; direkte IR-Testfixtures werden weiterhin geprüft. Compiler- und Integrationstests erfolgreich.
- [x] 3. Bytecode-Verifier: dekodierte Instruktionen für Operandenprüfung, Kontrollfluss und Ausführung wiederverwendet; separate vollständige Dekodierdurchläufe entfernt. Verifier-/Bytecode-Tests erfolgreich.
- [x] 4. ProgramImage: Konstruktorinvarianten beim Schreiben genutzt; Quellenindizes weitergereicht und String-Suchen durch einen Index ersetzt. Format-/Roundtrip-Tests und neue Kollisionsregression erfolgreich.
- [x] 5. ACP-Deklarationen: Schema-Kataloge erst bei benötigten ACP-Aggregaten angefordert; native Typfamilien je Checker-Kontext nur einmal registriert. Compiler-/Sema-/ACP-Tests erfolgreich.
- [x] 6. Direkte Abhängigkeiten: unbenutztes serde aus ACP/Debugger entfernt, socket2 auf Runtime-Tests beschränkt. `cargo check --workspace --all-targets --offline` erfolgreich.
- [x] 7. Transitive Abhängigkeiten: fancy-regex auf die bereits verwendete Version 0.19.2 vereinheitlicht und num_enum-Standardfeature entfernt. Workspace-Check und beide ACP-Interaktionstests erfolgreich.
- [x] 8. Entwicklungsbuild: Benchmark-Programm über `bench-tools` aus dem normalen Workspace-Build genommen. Der Cargo-Alias aktiviert es für Release; vollständige Rust-Tests verwenden `--all-features`. dev-fast-Workspace-Build erfolgreich, fpas-bench wurde nicht gebaut.
- [x] 9. Tests: Formatter-/LSP-Targets thematisch gebündelt, gemeinsame Helfer einmal je Suite eingebunden und doppelte Parser-/Formatter-Arbeit entfernt. Alle ursprünglichen 147/75 Fälle erhalten und erfolgreich ausgeführt; Prozessisolation bleibt bestehen.
- [x] 10. Compiler-Identität: bekannte ausgelagerte Testmodule und ausschließlich runtimebezogene Abhängigkeiten aus dem Artefakthash genommen. Neuer Test belegt stabile Identität bei Test-/Runtime-Änderungen und Invalidierung bei Compilerquellen, Compiler-Features und gemeinsamen Abhängigkeiten.
- [x] 11. Kleine Vereinfachungen: 17 Produktionsbefunde zu Kopien/Zwischenkollektionen bereinigt; fünf doppelte Registerlängenprüfungen durch die gemeinsame Invariante ersetzt. Runtime-/Debugger-Tests und gezielte Clippy-Prüfung erfolgreich.
- [x] Abschluss: Formatierung, Clippy, vollständige Rust-Tests und dev-fast-Programmbuild erfolgreich; Diff auf unbeabsichtigte Änderungen geprüft.

## Prüfplan

- Abhängigkeiten: Cargo-Auflösung, Produktionsgraphen, ACP-Interaktionstests.
- Verifier/IR/ProgramImage: vorhandene Compiler-, Bytecode-, Format- und Roundtrip-Tests; gezielte zusätzliche Regression nur bei einer bislang ungeschützten Invariante.
- ACP: bestehende Codec-/Binding-/Schema-Tests einschließlich Rohzahlen, unbekannter Felder, Duplikatschlüssel und Tiefengrenzen.
- Testbündelung: Fälle und Namen vor/nach dem Umbau vergleichen; gebündelte Targets ausführen.
- Build-Identität: bestehende Identitäts-/Cachetests um die veränderte Invalidierungsgrenze erweitern.
- Rust-Tests getrennt kompilieren und ausführen; bei umfangreichen Läufen den vorhandenen FPAS-Ressourcenwächter verwenden. Cargo-Parallelität bleibt unverändert.

## Umsetzung und Ergebnisse

- Alle elf Punkte sind umgesetzt und geprüft. Keine Commits oder Pushes erstellt.
- Abhängigkeiten: Workspace-Check und beide ACP-Interaktionstests erfolgreich.
- ProgramImage: Beim Verschieben der Prüfgrenze wird auch eine bislang erst beim Schreiben erkannte Pfadkollision bereits vom Konstruktor zurückgewiesen.
- Die neue ProgramImage-Regression schlug vor der Korrektur wie erwartet fehl.
- Test-Targets: Formatter 20 → 3, LSP 13 → 5, insgesamt 25 weniger zu linkende Testprogramme. Vorher/Nachher-Abgleich einschließlich Unit-Tests: 147/147 Formatter- und 75/75 LSP-Fälle; kein Fall fehlt und keiner ist versehentlich hinzugekommen.
- Compiler-Identität: explizite Liste ausgelagerter Testmodule sowie gefilterte Manifest-/Lockfile-Eingaben. Inline-Tests und unbekannte Quelldateien bleiben konservativ im Hash. Dafür nutzt das Buildskript den schon vorhandenen TOML-Parser; es kommen keine neuen Pakete ins Lockfile.
- `cargo check --workspace --all-features --all-targets --offline`: erfolgreich.
- `cargo build --workspace --locked --offline --profile dev-fast --timings`: erfolgreich in 3m 20s. Das ist die Dauer dieses Änderungsbuilds, keine Vorher-Nachher-Messung.
- `cargo test --workspace --all-features --locked --offline --no-run --timings`: erfolgreich; Ressourcenwächter meldet 62 Sekunden einschließlich Start/Abschluss.
- Clippy für alle Targets/Features mit `-D warnings`: erfolgreich. Zusätzlicher Produktionslauf mit `redundant_clone` und `needless_collect` ebenfalls ohne Warnungen.
- `cargo fmt --all -- --check` und `git diff --check`: erfolgreich.
- `cargo test --workspace --all-features --locked --offline --no-fail-fast`: erfolgreich, 5.718 bestanden, 0 fehlgeschlagen, 12 als ignoriert markiert. Diese sind isolierte Kindprozess-/Terminal-Fixtures und ein manueller Profiling-Test. Der Ressourcenwächter meldet 18m 47s und Exitcode 0.

## Lokale Prüfprotokolle

Die vollständigen, nicht versionierten Protokolle liegen unter `target/`:
`rust-debloat-dev-fast.log`, `rust-debloat-test-build-final.log`,
`rust-debloat-tests-final.log`, `rust-debloat-clippy-final.log` und
`rust-debloat-production-final.log`. Build-/Laufzeitgewinne sind noch nicht
vergleichend gemessen; belegt sind die entfernte Arbeit, die kleineren
Abhängigkeitsgraphen und die verringerte Zahl der Testprogramme.
