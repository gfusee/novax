pub trait IntoFilterTerms {
    fn into_filter_terms(self) -> Vec<(Vec<u8>, u32)>; // u32 is the position in topics.
}