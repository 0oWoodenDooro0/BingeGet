use ferrous_opencc::config::BuiltinConfig;
use ferrous_opencc::OpenCC;
use std::sync::OnceLock;

static S2T_CONVERTER: OnceLock<Option<OpenCC>> = OnceLock::new();
static T2S_CONVERTER: OnceLock<Option<OpenCC>> = OnceLock::new();

fn get_s2t() -> Option<&'static OpenCC> {
    S2T_CONVERTER
        .get_or_init(|| OpenCC::from_config(BuiltinConfig::S2t).ok())
        .as_ref()
}

fn get_t2s() -> Option<&'static OpenCC> {
    T2S_CONVERTER
        .get_or_init(|| OpenCC::from_config(BuiltinConfig::T2s).ok())
        .as_ref()
}

/// Normalizes full-width ASCII characters, full-width spaces, and converts to lowercase.
pub fn normalize_basic(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            // Full-width space (U+3000) -> standard space
            '\u{3000}' => out.push(' '),
            // Full-width ASCII variants (U+FF01 to U+FF5E) -> standard ASCII (U+0021 to U+007E)
            '\u{FF01}'..='\u{FF5E}' => {
                if let Some(ascii) = char::from_u32(c as u32 - 0xFEE0) {
                    out.push(ascii);
                } else {
                    out.push(c);
                }
            }
            other => out.push(other),
        }
    }
    out.to_lowercase()
}

/// Matches needle against haystack with:
/// - Case insensitivity
/// - Full-width to half-width ASCII conversion
/// - Whitespace tolerance
/// - Simplified <-> Traditional Chinese bidirectional variants matching
pub fn match_keyword(haystack: &str, needle: &str) -> bool {
    let needle_norm = normalize_basic(needle.trim());
    if needle_norm.is_empty() {
        return false;
    }

    let haystack_norm = normalize_basic(haystack);

    // 1. Direct normalized substring check
    if haystack_norm.contains(&needle_norm) {
        return true;
    }

    // 2. Space-insensitive check
    let needle_no_space = needle_norm.replace(' ', "");
    let haystack_no_space = haystack_norm.replace(' ', "");
    if !needle_no_space.is_empty() && haystack_no_space.contains(&needle_no_space) {
        return true;
    }

    // 3. Simplified -> Traditional check
    if let Some(s2t) = get_s2t() {
        let needle_t = s2t.convert(&needle_norm);
        let haystack_t = s2t.convert(&haystack_norm);
        if haystack_t.contains(&needle_t) {
            return true;
        }
        let needle_t_no_space = needle_t.replace(' ', "");
        let haystack_t_no_space = haystack_t.replace(' ', "");
        if !needle_t_no_space.is_empty() && haystack_t_no_space.contains(&needle_t_no_space) {
            return true;
        }
    }

    // 4. Traditional -> Simplified check
    if let Some(t2s) = get_t2s() {
        let needle_s = t2s.convert(&needle_norm);
        let haystack_s = t2s.convert(&haystack_norm);
        if haystack_s.contains(&needle_s) {
            return true;
        }
        let needle_s_no_space = needle_s.replace(' ', "");
        let haystack_s_no_space = haystack_s.replace(' ', "");
        if !needle_s_no_space.is_empty() && haystack_s_no_space.contains(&needle_s_no_space) {
            return true;
        }
    }

    false
}
