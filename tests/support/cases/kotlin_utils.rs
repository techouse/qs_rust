use super::CaseMeta;

pub(crate) fn internal_case_meta() -> Vec<CaseMeta> {
    vec![
        CaseMeta::new(
            "kotlin-qskotlin",
            "UtilsSpec.kt",
            "overflow append and mixed-key index tracking",
            "overflow internals",
            false,
        ),
        CaseMeta::new(
            "kotlin-qskotlin",
            "UtilsSpec.kt",
            "compact removes undefined entries from nested structures",
            "compact internals",
            false,
        ),
        CaseMeta::new(
            "kotlin-qskotlin",
            "UtilsSpec.kt",
            "sparse normalization preserves numeric and named overflow keys",
            "overflow internals",
            false,
        ),
    ]
}
