# Using text and keyboard together

## Using text and keyboard together

- Use **`Read` / `ReadLn`** for typed input and pipes (line discipline).
- Use **`ReadKey` / `ReadKeyEvent`** for games or immediate key handling.
- Do not assume that mixing the two in one tight loop will interleave predictably without designing your loop; they are different subsystems.

---

## See also

- [Console overview](README.md)
- [Quick reference](README.md#quick-reference)
