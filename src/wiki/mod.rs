//! Mirror of `mw.lojban.org` articles surfaced as the `wiki` source for `/waves`.
//!
//! - Storage: `wiki_articles` (see `migrations/V100__create_wiki_articles.sql`).
//! - Background sync: [`importer::sync_on_startup`] / [`importer::run_incremental_sync`] /
//!   [`importer::import_revision_histories`], wired in `src/background/service.rs`.
//! - Search/list helpers consumed by [`crate::waves`] live in [`service`].

pub mod controller;
pub mod dto;
pub mod importer;
pub mod markdown;
pub mod models;
pub mod service;

use actix_web::web;

/// Local username suffix for imported mw.lojban.org editors (`name@mw.lojban.org`).
pub const MW_USERNAME_SUFFIX: &str = "@mw.lojban.org";

/// `users.username` is `varchar(64)`.
const USERNAME_MAX_LEN: usize = 64;

/// Lensisku username for an imported MediaWiki editor, if it fits in `users.username`.
pub fn mw_import_username(mw_user: &str) -> Option<String> {
    let name = mw_user.trim();
    if name.is_empty() {
        return None;
    }
    let local = format!("{name}{MW_USERNAME_SUFFIX}");
    if local.chars().count() > USERNAME_MAX_LEN {
        return None;
    }
    Some(local)
}

/// MediaWiki account name, if `username` is an imported mw.lojban.org identity.
pub fn mw_account_name(username: &str) -> Option<&str> {
    username.strip_suffix(MW_USERNAME_SUFFIX).filter(|s| !s.is_empty())
}

/// mw.lojban.org user-page URL for an imported editor (not a lensisku profile).
pub fn mw_user_page_url(mw_account: &str) -> Option<String> {
    let name = mw_account.trim();
    if name.is_empty() {
        return None;
    }
    let slug = name.replace(' ', "_");
    Some(format!(
        "https://mw.lojban.org/papri/User:{}",
        urlencoding::encode(&slug)
    ))
}

/// Profile URL when `username` is a `*@mw.lojban.org` stub.
pub fn author_url_for_username(username: &str) -> Option<String> {
    mw_account_name(username).and_then(mw_user_page_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mw_import_username_adds_suffix() {
        assert_eq!(
            mw_import_username("Nick Nicholas").as_deref(),
            Some("Nick Nicholas@mw.lojban.org")
        );
        assert_eq!(mw_import_username("  "), None);
    }

    #[test]
    fn author_url_strips_suffix() {
        assert_eq!(
            author_url_for_username("Nick Nicholas@mw.lojban.org").as_deref(),
            Some("https://mw.lojban.org/papri/User:Nick_Nicholas")
        );
        assert_eq!(author_url_for_username("localuser"), None);
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/wiki").service(controller::get_wiki_article));
}
