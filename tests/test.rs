use language_tokenizer::*;

const FUZZY_THRESHOLD: f64 = 0.8;

macro_rules! text_match_tests {
    (
        $(
            $name:ident {
                term: $term:expr,
                term_translation: $term_translation:expr,
                source_text: $source_text:expr,
                source_translation: $source_translation:expr,
                source_algorithm: $source_algorithm:expr,
                translation_algorithm: $translation_algorithm:expr,
                source_case_sensitivity: $source_case_sensitivity:expr,
                translation_case_sensitivity: $translation_case_sensitivity:expr,
                match_mode: $match_mode:expr,
                should_match: $should_match:expr,
                permissive_match: $permissive_match:expr,
            }
        ),* $(,)?
    ) => {
        $(
            #[test]
            fn $name() {
                let (term_tokens, term_translation_tokens) = (
                    tokenize(
                        $term,
                        $source_algorithm,
                        $source_case_sensitivity,
                    ).unwrap(),
                    tokenize(
                        $term_translation,
                        $translation_algorithm,
                        $translation_case_sensitivity
                    ).unwrap()
                );

                let (source_tokens, translation_tokens) = (
                    tokenize(
                        $source_text,
                        $source_algorithm,
                        $source_case_sensitivity,
                    ).unwrap(),
                    tokenize(
                        $source_translation,
                        $translation_algorithm,
                        $translation_case_sensitivity
                    ).unwrap()
                );

                let source_match = find_match(
                    &source_tokens,
                    &term_tokens,
                    $match_mode,
                    $permissive_match
                );

                let translation_match = find_match(
                    &translation_tokens,
                    &term_translation_tokens,
                    $match_mode,
                    $permissive_match,
                );

                assert_eq!(
                    source_match.is_some(), $should_match,
                    "Matching {source_tokens:?} and {term_tokens:?} failed",
                );

                assert_eq!(
                    translation_match.is_some(), $should_match,
                    "Matching {translation_tokens:?} and {term_translation_tokens:?} failed",
                );
            }
        )*
    };
}

text_match_tests! {
    exact_english_match {
        term: "climate change",
        term_translation: "climate change",
        source_text: "The effects of climate change are accelerating.",
        source_translation: "The effects of climate change are accelerating.",
        source_algorithm: Algorithm::English,
        translation_algorithm: Algorithm::English,
        source_case_sensitivity: false,
        translation_case_sensitivity: false,
        match_mode: MatchMode::Exact,
        should_match: true,
        permissive_match: false,
    },
    stemmed_english_match {
        term: "run fast",
        term_translation: "run fast",
        source_text: "She was running fast to catch the bus.",
        source_translation: "She was running fast to catch the bus.",
        source_algorithm: Algorithm::English,
        translation_algorithm: Algorithm::English,
        source_case_sensitivity: false,
        translation_case_sensitivity: false,
        match_mode: MatchMode::Exact,
        should_match: true,
        permissive_match: false,
    },
    fuzzy_english_match {
        term: "color theory",
        term_translation: "color theory",
        source_text: "This course focuses on colour theory fundamentals.",
        source_translation: "This course focuses on colour theory fundamentals.",
        source_algorithm: Algorithm::English,
        translation_algorithm: Algorithm::English,
        source_case_sensitivity: false,
        translation_case_sensitivity: false,
        match_mode: MatchMode::Fuzzy {threshold: FUZZY_THRESHOLD},
        should_match: true,
        permissive_match: false,
    },
    japanese_to_english_match {
        term: "人工知能",
        term_translation: "artificial intelligence",
        source_text: "人工知能は多くの分野で使われています。",
        source_translation: "Artificial intelligence is used in many fields.",
        source_algorithm: Algorithm::Japanese,
        translation_algorithm: Algorithm::English,
        source_case_sensitivity: false,
        translation_case_sensitivity: false,
        match_mode: MatchMode::Exact,
        should_match: true,
        permissive_match: false,
    },
    chinese_to_english_fuzzy_translation {
        term: "机器学习",
        term_translation: "machine learning",
        source_text: "机器学习正在改变世界。",
        source_translation: "Machine-learning techniques are changing the world.",
        source_algorithm: Algorithm::Chinese,
        translation_algorithm: Algorithm::English,
        source_case_sensitivity: false,
        translation_case_sensitivity: false,
        match_mode: MatchMode::Fuzzy {threshold: FUZZY_THRESHOLD},
        should_match: true,
        permissive_match: false,
    },
    term_not_present {
        term: "quantum computing",
        term_translation: "quantum computing",
        source_text: "Classical computers are widely used.",
        source_translation: "Classical computers are widely used.",
        source_algorithm: Algorithm::English,
        translation_algorithm: Algorithm::English,
        source_case_sensitivity: false,
        translation_case_sensitivity: false,
        match_mode: MatchMode::Fuzzy {threshold: FUZZY_THRESHOLD},
        should_match: false,
        permissive_match: false,
    },
    french_to_english_match {
        term: "apprentissage automatique",
        term_translation: "machine learning",
        source_text: "L'apprentissage automatique transforme de nombreux secteurs.",
        source_translation: "Machine learning is transforming many industries.",
        source_algorithm: Algorithm::French,
        translation_algorithm: Algorithm::English,
        source_case_sensitivity: false,
        translation_case_sensitivity: false,
        match_mode: MatchMode::Exact,
        should_match: true,
        permissive_match: false,
    },
    russian_to_english_match {
        term: "искусственный интеллект",
        term_translation: "artificial intelligence",
        source_text: "Искусственный интеллект развивается быстро.",
        source_translation: "Artificial intelligence is developing rapidly.",
        source_algorithm: Algorithm::Russian,
        translation_algorithm: Algorithm::English,
        source_case_sensitivity: false,
        translation_case_sensitivity: false,
        match_mode: MatchMode::Exact,
        should_match: true,
        permissive_match: false,
    },
    norwegian_to_english_match {
        term: "maskinlæring",
        term_translation: "machine learning",
        source_text: "Maskinlæring brukes i mange bransjer.",
        source_translation: "Machine learning is used in many industries.",
        source_algorithm: Algorithm::Norwegian,
        translation_algorithm: Algorithm::English,
        source_case_sensitivity: false,
        translation_case_sensitivity: false,
        match_mode: MatchMode::Exact,
        should_match: true,
        permissive_match: false,
    },
    indonesian_to_english_match {
        term: "pembelajaran mesin",
        term_translation: "machine learning",
        source_text: "Pembelajaran mesin mengubah cara kita bekerja.",
        source_translation: "Machine learning is changing how we work.",
        source_algorithm: Algorithm::Indonesian,
        translation_algorithm: Algorithm::English,
        source_case_sensitivity: false,
        translation_case_sensitivity: false,
        match_mode: MatchMode::Exact,
        should_match: true,
        permissive_match: false,
    },
    thai_to_english_match {
        term: "การเรียนรู้ของเครื่อง",
        term_translation: "machine learning",
        source_text: "การเรียนรู้ของเครื่องกำลังเปลี่ยนแปลงโลก",
        source_translation: "Machine learning is changing the world.",
        source_algorithm: Algorithm::Thai,
        translation_algorithm: Algorithm::English,
        source_case_sensitivity: false,
        translation_case_sensitivity: false,
        match_mode: MatchMode::Exact,
        should_match: true,
        permissive_match: false,
    },
    greek_to_english_match {
        term: "τεχνητή νοημοσύνη",
        term_translation: "artificial intelligence",
        source_text: "Η τεχνητή νοημοσύνη έχει πολλές εφαρμογές.",
        source_translation: "Artificial intelligence has many applications.",
        source_algorithm: Algorithm::Greek,
        translation_algorithm: Algorithm::English,
        source_case_sensitivity: false,
        translation_case_sensitivity: false,
        match_mode: MatchMode::Exact,
        should_match: true,
        permissive_match: false,
    },
    german_to_english_match {
        term: "künstliche Intelligenz",
        term_translation: "artificial intelligence",
        source_text: "Künstliche Intelligenz verändert unsere Gesellschaft.",
        source_translation: "Artificial intelligence is transforming our society.",
        source_algorithm: Algorithm::German,
        translation_algorithm: Algorithm::English,
        source_case_sensitivity: false,
        translation_case_sensitivity: false,
        match_mode: MatchMode::Exact,
        should_match: true,
        permissive_match: false,
    },
    german_fuzzy_match {
        term: "maschinelles Lernen",
        term_translation: "machine learning",
        source_text: "Maschinen-Lernen ist eine Schlüsseltechnologie.",
        source_translation: "Machine-learning is a key technology.",
        source_algorithm: Algorithm::German,
        translation_algorithm: Algorithm::English,
        source_case_sensitivity: false,
        translation_case_sensitivity: false,
        match_mode: MatchMode::Fuzzy {threshold: FUZZY_THRESHOLD},
        should_match: true,
        permissive_match: false,
    },
    lowercase_no_match {
        term: "Downtown",
        term_translation: "Даунтаун",
        source_text: "There are strange things going in the downtown.",
        source_translation: "Странные вещи творятся в деловом районе.",
        source_algorithm: Algorithm::English,
        translation_algorithm: Algorithm::Russian,
        source_case_sensitivity: true,
        translation_case_sensitivity: true,
        match_mode: MatchMode::Exact,
        should_match: false,
        permissive_match: false,
    },
    lowercase_match {
        term: "Downtown",
        term_translation: "Даунтаун",
        source_text: "There are strange things going in The Downtown.",
        source_translation: "Странные вещи творятся в Даунтауне.",
        source_algorithm: Algorithm::English,
        translation_algorithm: Algorithm::Russian,
        source_case_sensitivity: true,
        translation_case_sensitivity: true,
        match_mode: MatchMode::Exact,
        should_match: true,
        permissive_match: false,
    },
    permissive_match {
        term: "Downtown",
        term_translation: "Даунтаун",
        source_text: "There are strange things going in THE DOWNTOWN.",
        source_translation: "Странные вещи творятся в ДАУНТАУНЕ.",
        source_algorithm: Algorithm::English,
        translation_algorithm: Algorithm::Russian,
        source_case_sensitivity: true,
        translation_case_sensitivity: true,
        match_mode: MatchMode::Both {threshold: 0.85},
        should_match: true,
        permissive_match: true,
    },
}
