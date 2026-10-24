use bingeget::extractors::text_normalize::match_keyword;

#[test]
fn test_exact_match() {
    assert!(match_keyword("Doctor X", "doctor"));
    assert!(match_keyword("Doctor X", "Doctor"));
    assert!(match_keyword("Doctor X", "X"));
    assert!(!match_keyword("Doctor X", "Nurse"));
}

#[test]
fn test_simplified_and_traditional_variants() {
    // Traditional query on Traditional text
    assert!(match_keyword("繁花", "繁花"));
    // Simplified query on Traditional text
    assert!(match_keyword("繁花", "繁花"));
    // Simplified query with character differences: 医生 vs 醫生, 机智 vs 機智
    assert!(match_keyword("機智醫生生活", "机智医生"));
    assert!(match_keyword("機智醫生生活", "医生"));
    // Traditional query on Simplified text
    assert!(match_keyword("狂飙", "狂飆"));
    assert!(match_keyword("隐秘的角落", "隱秘"));
}

#[test]
fn test_fullwidth_ascii_and_space_normalization() {
    // Full-width ASCII "Ｄｏｃｔｏｒ" matching "Doctor X"
    assert!(match_keyword("Doctor X", "Ｄｏｃｔｏｒ"));
    // Full-width space
    assert!(match_keyword("Doctor X", "Doctor　X"));
    // Extra spaces around query
    assert!(match_keyword("Doctor X", "  doctor  "));
}
