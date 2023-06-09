use ammonia::Builder;
use std::collections::hash_set::HashSet;

pub mod our_date_time;
pub mod bool_wrapper;
pub mod issues_reported;
pub mod user;
pub mod user_status;
pub mod pagination;

pub fn clean_html(src: &str) -> String {
    Builder::default()
        .tags(HashSet::new())
        .clean(src)
        .to_string()
}
