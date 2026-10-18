/// Normalize optional page query values into `(page, per_page, offset)`.
pub fn page_window(
    page: Option<i64>,
    per_page: Option<i64>,
    default_per_page: i64,
    max_per_page: i64,
) -> (i64, i64, i64) {
    let page = page.unwrap_or(1).max(1);
    let per_page = per_page.unwrap_or(default_per_page).clamp(1, max_per_page);
    let offset = page.saturating_sub(1).saturating_mul(per_page);
    (page, per_page, offset)
}
