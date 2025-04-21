#[derive(Default, Debug, Clone)]
pub struct EventQueryOptions {
    pub from: Option<u64>,
    pub size: Option<u64>,
    pub timestamp: Option<TimestampOption>,
    pub sort: Option<QueryEventsSortOptions>,
}

#[derive(Debug, Clone)]
pub enum TimestampOption {
    GreaterThanOrEqual(u64),
    LowerThanOrEqual(u64),
    Between(u64, u64),
}

#[derive(Debug, Clone)]
pub enum SortOption {
    Ascending,
    Descending,
}

#[derive(Default, Debug, Clone)]
pub struct QueryEventsSortOptions {
    pub timestamp: Option<SortOption>
}

impl SortOption {
     pub fn as_elastic_search_term(&self) -> &str {
         match self {
             SortOption::Ascending => "asc",
             SortOption::Descending => "desc"
         }
     }
}