# language-tokenizer

## Overview

`language-tokenizer` is a convenience wrapper around various Unicode and Natural Language Processing libraries used for analyzing, segmenting and tokenizing text.

The main purpose of this library is to tokenize text and use it in matching.

For processing Indo-European languages as well as Arabic, Indonesian etc., it uses custom normalization algorithm combined with battle-tested Snowball stemmer.

For processing CJK languages it uses `lindera` crate, which provides multiple dictionaries for Chinese, Japanese and Korean **or** ICU dictionary segmentation.

For processing Southeast Asian languages, it uses ICU LSTM segmentation.

## Example

Tokenizing text is simple as:

```rust
use language_tokenizer::{tokenize, Algorithm};

let text = "that's someone who can rizz just like a skibidi! zoomer slang rocks, 67";
let tokens = tokenize(text, Algorithm::English, false).unwrap();

assert_eq!(tokens, vec!["that", "someon", "who", "can", "rizz", "just", "like", "a", "skibidi", "zoomer", "slang", "rock", "67"])
```

Matching text is also built-in.

```rust
use language_tokenizer::{MatchMode, Algorithm, find_match, tokenize};

let haystack = "that's someone who can rizz just like a skibidi! zoomer slang rocks, 67";
let needle = "like a skibidi";

let haystack = tokenize(haystack, Algorithm::English, false).unwrap();
let needle = tokenize(needle, Algorithm::English, false).unwrap();

assert!(find_match(&haystack, &needle, MatchMode::Exact, false).is_some());
```

### CJK dictionaries

Lindera-backed CJK tokenizers (`japanese-lindera`, `chinese-lindera`, `korean-lindera`) load their dictionary from a filesystem path at runtime by default - nothing is embedded in the binary unless you opt into one of the `*-embed-*` features. Point a language at a dictionary directory with `set_dictionary_path` before tokenizing text in that language:

```rust
use language_tokenizer::{set_dictionary_path, tokenize, Algorithm};
use std::path::PathBuf;

set_dictionary_path(Algorithm::Japanese, Some(PathBuf::from("/path/to/ipadic-neologd")));

let tokens = tokenize("日本語の文章です", Algorithm::Japanese, false).unwrap();
```

Passing `None` (or never calling `set_dictionary_path` for a language at all) falls back to that language's embedded dictionary, if the crate was built with a matching `*-embed-*` feature - otherwise `tokenize` returns `Error::NoDictionary` for that language.

Japanese can have more than one embedded dictionary compiled in at once - `japanese-lindera-embed-ipadic`, `-ipadic-neologd`, and `-unidic` are independent, additive features. Which one is actually used as the fallback is a runtime choice, made with `set_japanese_embedded_dictionary`:

```rust
use language_tokenizer::{set_japanese_embedded_dictionary, JapaneseDictionaryKind};

set_japanese_embedded_dictionary(JapaneseDictionaryKind::Unidic);
```

If it's never called, the best dictionary actually compiled in is picked automatically, preferring `ipadic-neologd` > `unidic` > `ipadic`.

The directory a path points at is a Lindera-built dictionary (`dict.trie`, `dict.words`, `dict.vals`, `matrix.mtx`, etc.) - not a raw MeCab dictionary source. Get one of:

- **Pre-built** - the [Lindera releases page](https://github.com/lindera/lindera/releases) publishes ready-to-use `lindera-<dictionary>-<version>.zip` archives (e.g. `lindera-ipadic-neologd-5.3.0.zip`, `lindera-cc-cedict-5.3.0.zip`, `lindera-ko-dic-5.3.0.zip`) for every tagged release - grab the version matching this crate's `lindera` dependency and unzip it. This is the easiest route and needs no build tooling.
- **Built yourself** - install [`lindera-cli`](https://github.com/lindera/lindera/tree/main/lindera-cli) (`cargo install lindera-cli`) and run its `build` subcommand against a raw dictionary source: [mecab-ipadic-neologd](https://github.com/neologd/mecab-ipadic-neologd) for Japanese's extended dictionary (plain [ipadic](https://github.com/lindera/lindera/tree/main/lindera-ipadic) and [unidic](https://github.com/lindera/lindera/tree/main/lindera-unidic) are also options), [CC-CEDICT-MeCab](https://github.com/lindera/CC-CEDICT-MeCab) for Chinese, and [mecab-ko-dic](https://bitbucket.org/eunjeon/mecab-ko-dic/) for Korean.

## Features

No tokenizer is available by default, you should opt-in everything manually with features.

- `snowball` - Enables tokenization for [all languages supported by Snowball](https://snowballstem.org/algorithms/).

- `japanese-lindera` - Enables Lindera-backed Japanese tokenization, loaded from a path set via `set_dictionary_path` (see above).
- `chinese-lindera` - Enables Lindera-backed Chinese tokenization (`cc-cedict` dictionary format), loaded from a path.
- `korean-lindera` - Enables Lindera-backed Korean tokenization (`ko-dic` dictionary format), loaded from a path.

- `japanese-icu` - Enables tokenization for Japanese using ICU's built-in segmentation data. No dictionary needed. Mutually exclusive with `japanese-lindera`.
- `chinese-icu` - Enables tokenization for Chinese using ICU's built-in segmentation data. No dictionary needed. Mutually exclusive with `chinese-lindera`. Korean has no ICU equivalent - `korean-lindera` is the only option.

- `japanese-lindera-embed-ipadic` / `japanese-lindera-embed-ipadic-neologd` / `japanese-lindera-embed-unidic` - Additive on top of `japanese-lindera`: bundles the named dictionary into the binary as a compile-time fallback for when no path is set at runtime. Any combination may be enabled at once - `set_japanese_embedded_dictionary` picks which one is used at runtime (see above). **`ipadic-neologd` alone adds well over a gigabyte to the binary and is slow to compile - reach for a runtime path, `ipadic`, or ICU instead unless you specifically need its slang/neologism coverage.**
- `chinese-lindera-embed` - Additive on top of `chinese-lindera`: bundles `cc-cedict`.
- `korean-lindera-embed` - Additive on top of `korean-lindera`: bundles `ko-dic`.

- `southeast-asian` - Enables tokenization for Southeast Asian languages, such as Burmese, Khmer, Lao, and Thai using LSTM.

- `full` - Shorthand for `snowball`, `japanese-lindera`, `chinese-lindera`, `korean-lindera`, `southeast-asian` - the smallest-binary CJK setup (path-loaded, nothing embedded). Add the `*-embed-*` features separately if you want a bundled fallback dictionary.

- `serde` - Some serialization/deserialization for types.

## License

Project is licensed under WTFPL.
