#![warn(clippy::all, clippy::pedantic)]
#![doc = include_str!("../README.md")]

#[cfg(any(
    feature = "southeast-asian",
    feature = "japanese-icu",
    feature = "chinese-icu"
))]
use icu_segmenter::{WordSegmenter, options::WordBreakInvariantOptions};
#[cfg(any(
    feature = "southeast-asian",
    feature = "japanese-icu",
    feature = "chinese-icu"
))]
use itertools::Itertools;
#[cfg(any(
    feature = "japanese-ipadic-neologd-lindera",
    feature = "japanese-ipadic-lindera",
    feature = "japanese-unidic-lindera",
    feature = "chinese-lindera",
    feature = "korean-lindera"
))]
use lindera::{
    dictionary::load_dictionary, mode::Mode, segmenter::Segmenter, tokenizer::Tokenizer,
};
use num_enum::{FromPrimitive, IntoPrimitive};
#[cfg(feature = "serde")]
use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeTuple,
    {Deserialize, Deserializer, Serialize, Serializer},
};
#[cfg(feature = "serde")]
use std::fmt;
#[cfg(feature = "snowball")]
use std::mem::transmute;
use strum_macros::Display;
use thiserror::Error;
#[cfg(feature = "snowball")]
use unicode_normalization::UnicodeNormalization;
#[cfg(feature = "snowball")]
use unicode_segmentation::UnicodeSegmentation;
#[cfg(feature = "snowball")]
use waken_snowball::{Algorithm as SnowballAlgorithm, stem};

#[cfg(all(
    feature = "japanese-ipadic-neologd-lindera",
    any(
        feature = "japanese-ipadic-lindera",
        feature = "japanese-unidic-lindera",
        feature = "japanese-icu",
    )
))]
compile_error!("Only one Japanese tokenizer feature may be enabled at a time.");

#[cfg(all(
    feature = "japanese-ipadic-lindera",
    any(
        feature = "japanese-ipadic-neologd-lindera",
        feature = "japanese-unidic-lindera",
        feature = "japanese-icu",
    )
))]
compile_error!("Only one Japanese tokenizer feature may be enabled at a time.");

#[cfg(all(
    feature = "japanese-unidic-lindera",
    any(
        feature = "japanese-ipadic-neologd-lindera",
        feature = "japanese-ipadic-lindera",
        feature = "japanese-icu",
    )
))]
compile_error!("Only one Japanese tokenizer feature may be enabled at a time.");

#[cfg(all(
    feature = "japanese-icu",
    any(
        feature = "japanese-ipadic-neologd-lindera",
        feature = "japanese-ipadic-lindera",
        feature = "japanese-unidic-lindera",
    )
))]
compile_error!("Only one Japanese tokenizer feature may be enabled at a time.");

#[cfg(all(feature = "chinese-lindera", feature = "chinese-icu"))]
compile_error!("Only one Chinese tokenizer feature may be enabled at a time.");

#[cfg(any(
    feature = "japanese-ipadic-neologd-lindera",
    feature = "japanese-ipadic-lindera",
    feature = "japanese-unidic-lindera",
    feature = "chinese-lindera",
    feature = "korean-lindera"
))]
thread_local! {
    static JAPANESE_TOKENIZER: Tokenizer =
        Tokenizer::new(Segmenter::new(
            Mode::Normal,
            load_dictionary(
                #[cfg(feature = "japanese-ipadic-neologd-lindera")]
                "embedded://ipadic-neologd",

                #[cfg(feature = "japanese-ipadic-lindera")]
                "embedded://ipadic",

                #[cfg(feature = "japanese-unidic-lindera")]
                "embedded://unidic",

                #[cfg(not(any(
                    feature = "japanese-ipadic-neologd-lindera",
                    feature = "japanese-ipadic-lindera",
                    feature = "japanese-unidic-lindera"
                )))]
                "",
            ).unwrap(),
            None,
        ));
    static KOREAN_TOKENIZER: Tokenizer =
        Tokenizer::new(Segmenter::new(
            Mode::Normal,
            load_dictionary("embedded://ko-dic").unwrap(),
            None,
        ));
    static CHINESE_TOKENIZER: Tokenizer =
        Tokenizer::new(Segmenter::new(
            Mode::Normal,
            load_dictionary("embedded://cc-cedict").unwrap(),
            None,
        ));
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, FromPrimitive, IntoPrimitive,
)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(into = "i8", try_from = "i8"))]
#[repr(i8)]
pub enum Algorithm {
    #[default]
    None = -1,

    Arabic,
    Armenian,
    Basque,
    Catalan,
    Danish,
    Dutch,
    DutchPorter,
    English,
    Esperanto,
    Estonian,
    Finnish,
    French,
    German,
    Greek,
    Hindi,
    Hungarian,
    Indonesian,
    Irish,
    Italian,
    Lithuanian,
    Lovins,
    Nepali,
    Norwegian,
    Porter,
    Portuguese,
    Romanian,
    Russian,
    Serbian,
    Spanish,
    Swedish,
    Tamil,
    Turkish,
    Yiddish,

    Japanese,
    Chinese,
    Korean,

    Thai,
    Burmese,
    Lao,
    Khmer,
}

impl Algorithm {
    pub const fn is_snowball(self) -> bool {
        !self.is_cjk() && !self.is_southeast_asian()
    }

    pub const fn is_cjk(self) -> bool {
        matches!(self, Self::Japanese | Self::Chinese | Self::Korean)
    }

    pub const fn is_southeast_asian(self) -> bool {
        matches!(self, Self::Thai | Self::Burmese | Self::Lao | Self::Khmer)
    }
}

#[derive(Debug, Error)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Error {
    #[error(
        "No tokenizer found for algorithm {0:?}, you might want to enable a crate feature that corresponds to desired language."
    )]
    NoTokenizer(Algorithm),
}

/// Specifies mode for matching text in [`match_text`] function.
///
/// # Variants
///
/// - [`MatchMode::Exact`] - tokens are matched for exact similarity.
/// - [`MatchMode::Fuzzy`] - tokens are matched fuzzily. This variant holds fuzzy match threshold as [`f64`].
/// - [`MatchMode::Exact`] - tokens are matches for exact similarity, and if match failed, tokens are matched fuzzily. This variant holds fuzzy match threshold as [`f64`].
///
/// # Note
///
/// Threshold should be in range of 0.0 and 1.0. Adjust threshold for your use case. Generally thresholds above 0.7-0.75 are fine, and generally you should use higher thresholds for smaller inputs.
///
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MatchMode {
    Exact,
    Fuzzy { threshold: f64 },
    Both { threshold: f64 },
}

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub start: u32, // char offset in original input string
    pub len: u32,   // char length in original input string
}

impl<T> PartialEq<T> for Token
where
    T: AsRef<str>,
{
    fn eq(&self, other: &T) -> bool {
        self.text == other.as_ref()
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

impl Eq for Token {}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum MatchResult {
    /// Exact match result, containing match offset position in haystack, and match length in characters.
    Exact { offset: u32, len: u32 },
    /// Fuzzy match result, containing match offset position in haystack, match length in characters, and match score as [`f64`].
    Fuzzy { offset: u32, len: u32, score: f64 },
}

#[cfg(feature = "serde")]
impl Serialize for MatchResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            MatchResult::Exact((a, b)) => (a, b).serialize(serializer),
            MatchResult::Fuzzy((a, b), score) => (a, b, score).serialize(serializer),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for MatchResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MatchResultVisitor;

        impl<'de> Visitor<'de> for MatchResultVisitor {
            type Value = MatchResult;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a tuple of length 2 or 3")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let a: usize = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let b: usize = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                if let Some(score) = seq.next_element::<f64>()? {
                    Ok(MatchResult::Fuzzy((a, b), score))
                } else {
                    Ok(MatchResult::Exact((a, b)))
                }
            }
        }

        deserializer.deserialize_seq(MatchResultVisitor)
    }
}

#[cfg(feature = "serde")]
impl Serialize for MatchMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(2)?;
        match *self {
            MatchMode::Exact => {
                tup.serialize_element(&0u8)?;
                tup.serialize_element(&0.0f64)?;
            }
            MatchMode::Fuzzy(v) => {
                tup.serialize_element(&1u8)?;
                tup.serialize_element(&v)?;
            }
            MatchMode::Both(v) => {
                tup.serialize_element(&2u8)?;
                tup.serialize_element(&v)?;
            }
        }
        tup.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for MatchMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MatchModeVisitor;

        impl<'de> Visitor<'de> for MatchModeVisitor {
            type Value = MatchMode;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a tuple [u8, f64]")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let tag: u8 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                let value: f64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                match tag {
                    0 => Ok(MatchMode::Exact),
                    1 => Ok(MatchMode::Fuzzy(value)),
                    2 => Ok(MatchMode::Both(value)),
                    _ => Err(de::Error::custom(format!("invalid MatchMode tag: {}", tag))),
                }
            }
        }

        deserializer.deserialize_tuple(2, MatchModeVisitor)
    }
}

#[cfg(feature = "snowball")]
fn normalize_punctuation(s: &str) -> String {
    s.chars()
        .map(|c| match c as u32 {
            0x2010..=0x2015 => '\'',
            0x201C..=0x201F => '"',
            0x2018..=0x201B => '-',
            _ => c,
        })
        .collect()
}

#[cfg(feature = "snowball")]
fn tokenize_snowball(text: &str, algorithm: Algorithm, case_sensitive: bool) -> Vec<Token> {
    let mut tokens = Vec::new();

    // Iterate over ORIGINAL text with byte indices
    for (byte_start, word) in text.unicode_word_indices() {
        let trimmed = word.trim_matches('\'');

        if !trimmed.chars().any(|c| c.is_alphabetic() || c.is_numeric()) {
            continue;
        }

        // Compute character offsets safely
        let start = text[..byte_start].chars().count();
        let len = trimmed.chars().count();

        // Normalize + stem ONLY the token text
        let normalized: String = trimmed.nfkc().collect();
        let normalized = normalize_punctuation(&normalized);

        let token_text = if case_sensitive {
            stem(
                unsafe { transmute::<Algorithm, SnowballAlgorithm>(algorithm) },
                &normalized,
            )
            .into_owned()
        } else {
            stem(
                unsafe { transmute::<Algorithm, SnowballAlgorithm>(algorithm) },
                &normalized.to_lowercase(),
            )
            .into_owned()
        };

        tokens.push(Token {
            text: token_text,
            start: start as u32,
            len: len as u32,
        });
    }

    tokens
}

#[cfg(any(
    feature = "japanese-ipadic-neologd-lindera",
    feature = "japanese-ipadic-lindera",
    feature = "japanese-unidic-lindera",
    feature = "chinese-lindera",
    feature = "korean-lindera",
    feature = "japanese-icu",
    feature = "chinese-icu"
))]
fn tokenize_cjk(text: &str, algorithm: Algorithm) -> Vec<Token> {
    match algorithm {
        Algorithm::Chinese => {
            #[cfg(feature = "chinese-lindera")]
            {
                CHINESE_TOKENIZER.with(|t| {
                    t.tokenize(text)
                        .unwrap()
                        .into_iter()
                        .map(|tok| {
                            let start = text[..tok.byte_start].chars().count();
                            let len = tok.surface.chars().count();

                            Token {
                                text: tok.surface.into_owned(),
                                start: start as u32,
                                len: len as u32,
                            }
                        })
                        .collect()
                })
            }

            #[cfg(feature = "chinese-icu")]
            tokenize_cjk_icu(text, algorithm)
        }

        Algorithm::Japanese => {
            #[cfg(any(
                feature = "japanese-ipadic-neologd-lindera",
                feature = "japanese-ipadic-lindera",
                feature = "japanese-unidic-lindera",
            ))]
            {
                JAPANESE_TOKENIZER.with(|t| {
                    t.tokenize(text)
                        .unwrap()
                        .into_iter()
                        .map(|tok| {
                            let start = text[..tok.byte_start].chars().count();
                            let len = tok.surface.chars().count();

                            Token {
                                text: tok.surface.into_owned(),
                                start: start as u32,
                                len: len as u32,
                            }
                        })
                        .collect()
                })
            }

            #[cfg(feature = "japanese-icu")]
            tokenize_cjk_icu(text, algorithm)
        }

        Algorithm::Korean =>
        {
            #[cfg(feature = "korean-lindera")]
            KOREAN_TOKENIZER.with(|t| {
                t.tokenize(text)
                    .unwrap()
                    .into_iter()
                    .map(|tok| {
                        let start = text[..tok.byte_start].chars().count();
                        let len = tok.surface.chars().count();

                        Token {
                            text: tok.surface.into_owned(),
                            start: start as u32,
                            len: len as u32,
                        }
                    })
                    .collect()
            })
        }

        _ => unreachable!(),
    }
}

#[cfg(any(feature = "japanese-icu", feature = "chinese-icu"))]
fn tokenize_cjk_icu(text: &str, _algorithm: Algorithm) -> Vec<Token> {
    let segmenter = WordSegmenter::new_auto(WordBreakInvariantOptions::default());

    segmenter
        .segment_str(text)
        .tuple_windows()
        .map(|(i, j)| {
            let slice = &text[i..j];

            Token {
                text: slice.to_owned(),
                start: text[..i].chars().count() as u32,
                len: slice.chars().count() as u32,
            }
        })
        .collect()
}

#[cfg(feature = "southeast-asian")]
fn tokenize_southeast_asian(text: &str, _algorithm: Algorithm) -> Vec<Token> {
    let segmenter = WordSegmenter::new_lstm(WordBreakInvariantOptions::default());

    segmenter
        .segment_str(text)
        .tuple_windows()
        .map(|(i, j)| {
            let slice = &text[i..j];

            Token {
                text: slice.to_owned(),
                start: text[..i].chars().count() as u32,
                len: slice.chars().count() as u32,
            }
        })
        .collect()
}

/// Tokenizes text to a [`Vec`] of [`Token`]s.
///
/// # Parameters
///
/// - `text` - text to tokenize.
/// - `algorithm` - algorithm to use.
/// - `case_sensitive` - lowercase all tokens or not. Only for non-CJK and non Southeast Asian algorithms.
///
/// # Returns
///
/// - [`Vec<Token>`] if tokenizer for `algorithm` was found.
/// - [`Error`] otherwise.
///
/// # Errors
///
/// - [`Error::NoTokenizer`] - no tokenizer was found. No tokenizers are enabled by default, you need to explicitly enable the desired ones with cargo features.
///
/// # Example
///
/// ```
/// use language_tokenizer::{tokenize, Algorithm};
///
/// let text = "that's someone who can rizz just like a skibidi! zoomer slang rocks, 67";
/// let tokens = tokenize(text, Algorithm::English, false).unwrap();
///
/// assert_eq!(tokens, vec!["that", "someon", "who", "can", "rizz", "just", "like", "a", "skibidi", "zoomer", "slang", "rock", "67"])
/// ```
///
pub fn tokenize(
    text: &str,
    algorithm: Algorithm,
    case_sensitive: bool,
) -> Result<Vec<Token>, Error> {
    if algorithm.is_snowball() {
        #[cfg(feature = "snowball")]
        return Ok(tokenize_snowball(text, algorithm, case_sensitive));
    } else if algorithm.is_cjk() {
        #[cfg(any(
            feature = "japanese-ipadic-neologd-lindera",
            feature = "japanese-ipadic-lindera",
            feature = "japanese-unidic-lindera",
            feature = "chinese-lindera",
            feature = "korean-lindera",
            feature = "japanese-icu",
            feature = "chinese-icu"
        ))]
        return Ok(tokenize_cjk(text, algorithm));
    } else if algorithm.is_southeast_asian() {
        #[cfg(feature = "southeast-asian")]
        return Ok(tokenize_southeast_asian(text, algorithm));
    }

    Err(Error::NoTokenizer(algorithm))
}

fn find_exact_match(haystack: &[Token], needle: &[Token], permissive: bool) -> Option<MatchResult> {
    haystack.windows(needle.len()).find_map(|window| {
        let matches = if permissive {
            window.iter().zip(needle).all(|(a, b)| {
                let a_lower = a.text.to_lowercase();
                let b_lower = b.text.to_lowercase();

                if a_lower == b_lower {
                    let a_upper_count = a.text.chars().filter(|c| c.is_uppercase()).count();
                    let b_upper_count = b.text.chars().filter(|c| c.is_uppercase()).count();

                    a_upper_count >= b_upper_count
                } else {
                    false
                }
            })
        } else {
            window == needle
        };

        matches.then_some(MatchResult::Exact {
            offset: window[0].start,
            len: needle.iter().fold(0, |mut acc, a| {
                acc += a.len;
                acc
            }),
        })
    })
}

fn find_fuzzy_match(
    haystack: &[Token],
    needle: &[Token],
    threshold: f64,
    permissive: bool,
    _collapse: bool,
) -> Option<MatchResult> {
    haystack.windows(needle.len()).find_map(|window| {
        let score = window
            .iter()
            .zip(needle)
            .map(|(a, b)| {
                if permissive {
                    strsim::normalized_levenshtein(&a.text.to_lowercase(), &b.text.to_lowercase())
                } else {
                    strsim::normalized_levenshtein(&a.text, &b.text)
                }
            })
            .sum::<f64>()
            / needle.len() as f64;

        let passes_threshold = if score >= threshold && permissive {
            window.iter().zip(needle).all(|(a, b)| {
                let a_upper_count = a.text.chars().filter(|c| c.is_uppercase()).count();
                let b_upper_count = b.text.chars().filter(|c| c.is_uppercase()).count();

                a_upper_count >= b_upper_count
            })
        } else {
            score >= threshold
        };

        passes_threshold.then_some(MatchResult::Fuzzy {
            offset: window[0].start,
            len: window.iter().fold(0, |mut acc, a| {
                acc += a.len;
                acc
            }),
            score,
        })
    })
}

/// Matches two [`Vec`]s of tokens based on [`MatchMode`] and returns the first match.
///
/// # Parameters
///
/// - `haystack` - haystack to seek.
/// - `needle` - needle to match.
/// - `mode` - [`MatchMode`] to use for matching. See the enum for more info.
/// - `permissive` - If `haystack` is more uppercased than `needle`, they will still match.
///
/// # Returns
///
/// - [`MatchResult`] if match is found.
/// - [`None`] otherwise.
///
/// # Example
///
/// ```
/// use language_tokenizer::{MatchMode, Algorithm, find_match, tokenize};
///
/// let haystack = "that's someone who can rizz just like a skibidi! zoomer slang rocks, 67";
/// let needle = "like a skibidi";
///
/// let haystack = tokenize(haystack, Algorithm::English, false).unwrap();
/// let needle = tokenize(needle, Algorithm::English, false).unwrap();
///
/// assert!(find_match(&haystack, &needle, MatchMode::Exact, false).is_some());
/// ```
///
pub fn find_match(
    haystack: &[Token],
    needle: &[Token],
    mode: MatchMode,
    permissive: bool,
) -> Option<MatchResult> {
    if needle.len() == 0 || needle.len() > haystack.len() {
        return None;
    }

    match mode {
        MatchMode::Exact => find_exact_match(&haystack, &needle, permissive),
        MatchMode::Fuzzy { threshold } => {
            find_fuzzy_match(&haystack, &needle, threshold, permissive, false)
        }
        MatchMode::Both { threshold } => find_exact_match(&haystack, &needle, permissive)
            .or_else(|| find_fuzzy_match(&haystack, &needle, threshold, permissive, false)),
    }
}

/// Matches two [`Vec`]s of tokens based on [`MatchMode`] and returns all matches.
///
/// # Parameters
///
/// - `haystack` - haystack to seek.
/// - `needle` - needle to match.
/// - `mode` - [`MatchMode`] to use for matching. See the enum for more info.
/// - `permissive` - If `haystack` is more uppercased than `needle`, they will still match.
///
/// # Returns
///
/// - [`Vec`] of [MatchResult]s. If no matches were found, it is empty.
///
/// # Example
///
/// ```
/// use language_tokenizer::{MatchMode, Algorithm, find_match, tokenize};
///
/// let haystack = "that's someone who can rizz just like a skibidi! zoomer slang rocks, 67";
/// let needle = "like a skibidi";
///
/// let haystack = tokenize(haystack, Algorithm::English, false).unwrap();
/// let needle = tokenize(needle, Algorithm::English, false).unwrap();
///
/// assert!(find_match(&haystack, &needle, MatchMode::Exact, false).is_some());
/// ```
///
pub fn find_all_matches(
    haystack: &[Token],
    needle: &[Token],
    mode: MatchMode,
    permissive: bool,
) -> Vec<MatchResult> {
    if needle.len() == 0 || needle.len() > haystack.len() {
        return Vec::new();
    }

    let mut results = Vec::new();
    let mut offset = 0u32;

    while offset < haystack.len() as u32 {
        let slice = &haystack[offset as usize..];
        let found = find_match(slice, needle, mode, permissive);

        match found {
            Some(t) => {
                match t {
                    MatchResult::Exact { offset: start, .. } => {
                        let absolute_start = offset + start;
                        offset = absolute_start + 1;
                    }
                    MatchResult::Fuzzy { offset: start, .. } => {
                        let absolute_start = offset + start;
                        offset = absolute_start + 1;
                    }
                }

                results.push(t);
            }
            None => break,
        }
    }

    results
}
