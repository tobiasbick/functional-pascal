# Tokenizer reference vectors

This optional FPAS helper regenerates the checked-in native tokenizer vectors
using Python `tiktoken` as an independent oracle. Python is not needed by the
tokenizer API, the agent-budget tool or normal tests.

Install `tiktoken` into the Python environment selected for this validation.
Download the upstream `o200k_base.tiktoken` rank file and verify its published
SHA-256:

`446a9538cb6c348e3516120d7c08b09f57c36495e2acfffe59a5bf8b0cfb1a2d`

Source: the `o200k_base` factory in
[OpenAI tiktoken](https://github.com/openai/tiktoken/blob/main/tiktoken_ext/openai_public.py).
The rank file may be obtained from the matching
[tiktoken-rs asset](https://github.com/zurawiki/tiktoken-rs/blob/main/tiktoken-rs/assets/o200k_base.tiktoken).
Keep downloaded files under `.temp-data`.

From the repository root:

~~~text
fpas run tools/tokenizer-reference/tokenizer-reference.fpasprj -- python .temp-data/tokenizer/o200k_base.tiktoken crates/fpas-runtime/src/execution/vm/hosted/tokenizer/reference.json
~~~

The helper supplies the corpus on stdin and overrides only the upstream
factory's rank-file loader to use the verified local file. Pattern construction,
BPE and ordinary-text token IDs come from the Python library. No reference
network request is required.

The corpus covers case transitions, combining marks, multilingual scripts,
emoji/ZWJ sequences, control characters, Unicode whitespace, digit grouping,
literal special-token markers and repeated text. The generated file records
the reference version and encoding. Review and rerun the wrapper regressions
after regeneration.
