# Security Report: Backend (`src/`) Analysis

**Scope:** Full analysis of `/home/user/lojban/lensisku/src/` (Rust backend).  
**Areas:** Vulnerabilities, penetration possibilities, rate limiting, SQL injection, ACL/authorization, and related issues.  
**Date:** 2026-03-15.

---

## 1. Executive Summary

The backend uses parameterized SQL consistently, has role-based access with a permission cache, and applies rate limiting on auth-sensitive endpoints. Several issues require attention: weak legacy password hashing, over-permissive or missing ACL on some routes, CORS and export routes exposed without auth, and information leakage in error responses. Recommended fixes are listed per finding.

---

## 2. Vulnerabilities

### 2.1 Authentication & Password Storage

| ID | Severity | Finding |
|----|----------|--------|
| **AUTH-1** | **High** | **Legacy MD5 + ROT13 password support.** In `auth/service.rs`, `verify_password()` accepts stored hashes that are 32 hex characters (MD5). For those, the password is ROT13’ed then MD5’ed. MD5 is cryptographically broken and ROT13 adds no security. **Risk:** Credential theft and offline cracking if the password table is leaked. |
| **AUTH-2** | **Low** | **Panic on missing env in auth.** `auth/service.rs` uses `.expect("JWT_SECRET must be set")` and similar for `REFRESH_TOKEN_SECRET`, `FRONTEND_URL`, `GOOGLE_*`. If any are unset, the process panics. Prefer returning a config error at startup instead of panicking in request path. |

### 2.2 Authorization (ACL)

| ID | Severity | Finding |
|----|----------|--------|
| **ACL-1** | **Medium** | **`GET /auth/permissions` has no permission check.** The handler `get_permissions` in `auth/controller.rs` is under the auth scope but has no `#[protect(...)]`. OpenAPI documents `manage_roles`; any authenticated user can list all system permissions. **Fix:** Add `#[protect(any("manage_roles"))]` (or the intended permission) and keep OpenAPI in sync. |
| **ACL-2** | **Medium** | **`GET /users` (list users) is unauthenticated.** `users/mod.rs` registers `list_users` on the public `web::scope("/users")` (no bearer scope). Anyone can paginate and filter users and see obfuscated email, username, realname, role, etc. **Fix:** If this is admin-only, wrap the route in the bearer scope and add `#[protect(...)]` for a permission like `manage_users` or `list_users`. If it is intentionally public, document it and consider reducing fields (e.g. no email at all). |
| **ACL-3** | **Medium** | **Cached export endpoints are unauthenticated.** In `export/mod.rs`, `list_cached_exports` and `download_cached_export` are registered on the bare `web::scope("export")` with no `HttpAuthentication::bearer`. OpenAPI marks them as `bearer_auth`. Unauthenticated users can list and download cached dictionary exports. **Fix:** Either protect both with the same bearer scope as `export_dictionary`, or clearly document that cached exports are public. |
| **ACL-4** | **Medium** | **Assistant `/assistant/chat` has no auth.** `assistant/mod.rs` configures the chat handler without authentication. Anyone can send requests and consume the OpenRouter (or similar) API and backend resources. **Fix:** Require authentication (e.g. wrap in bearer scope) and optionally rate-limit per user. |
| **ACL-5** | **Low** | **`block_user` and `assign_role` rely only on service-layer checks.** Handlers do not use `#[protect(...)]`; the auth scope ensures the user is authenticated and the service checks `block_users` / `manage_roles`. This is functionally correct but inconsistent with other admin endpoints that use `#[protect]`. **Fix:** Add `#[protect(any("block_users"))]` and `#[protect(any("manage_roles"))]` respectively for clarity and defense in depth. |

### 2.3 Rate Limiting

| ID | Severity | Finding |
|----|----------|--------|
| **RATE-1** | **Low** | **Global rate limiter key is IP + path.** `middleware/mod.rs` uses `format!("{}:{}", ip, path)`. A single IP can still send 100 req/min per path; aggressive crawlers or scripts can spread load across paths. Consider a global per-IP cap or stricter limits on expensive endpoints. |
| **RATE-2** | **Info** | **Login limiter:** 20 attempts per 15 minutes per IP; 5 failures per identifier (username/email) per 15 minutes. **Password reset:** 1 request per 4 hours per email. **Email confirmation resend:** 5 attempts per 30 minutes with exponential backoff. These are reasonable; ensure Redis is available so limiters do not fail open. |

### 2.4 SQL Injection

| ID | Severity | Finding |
|----|----------|--------|
| **SQL-1** | **None** | **Parameterized queries.** All observed DB access uses `$1`, `$2`, etc. with bound parameters. Dynamic fragments (e.g. in `comments/service.rs` and `collections/service.rs`) are built from fixed strings and numeric param indices (e.g. `format!("... = ${}", param_count)`); user input is only passed as parameters, not concatenated into SQL. **No SQL injection identified.** |

### 2.5 Input Validation & Injection

| ID | Severity | Finding |
|----|----------|--------|
| **INP-1** | **Low** | **Redis keys from user input.** In `middleware/limiter.rs`, login and email limiters use `format!("login_attempts:{}", ip)` and `format!("...:{}", email)` (or identifier). If `ip` or `email` contained e.g. `\r\n` or very long strings, Redis key semantics could be abused or cause performance issues. **Fix:** Sanitize/normalize (e.g. max length, strip control chars) or use a safe encoding for key segments. |
| **INP-2** | **Low** | **Content-Disposition filename.** In `export/controller.rs`, `filename` from the service is used in `format!("attachment; filename=\"{}\"", filename)`. If `filename` ever came from user-controlled or untrusted DB content, it could contain `"`, `\`, or newlines and lead to header injection or response splitting. **Fix:** Sanitize (e.g. strip or escape `"`, `\`, `\r`, `\n`) or use a fixed pattern (e.g. `dictionary-{tag}.{ext}`) for cached exports. |
| **INP-3** | **Info** | **HTML sanitization.** Profile and collection text (e.g. realname, url, personal, collection name/description) are sanitized with `remove_html_tags` (Ammonia with no tags allowed) in `auth/service.rs` and `collections/service.rs`, reducing XSS in stored data. |

### 2.6 Information Disclosure

| ID | Severity | Finding |
|----|----------|--------|
| **INFO-1** | **Medium** | **Refresh token error details.** In `auth/controller.rs`, on invalid refresh token the response includes `"details": e.to_string()`. Internal error messages may leak stack or implementation details. **Fix:** Return a generic message (e.g. "Invalid refresh token") and log the real error server-side. |
| **INFO-2** | **Medium** | **Debug-style error in comments.** In `comments/controller.rs`, several handlers return `"details": format!("{:#?}", e)`, exposing debug representation of errors to the client. **Fix:** Return a stable, safe message; log the full error server-side. |
| **INFO-3** | **Low** | **Database/error strings in other handlers.** Various controllers use `e.to_string()` or `format!("... {}", e)` in responses (e.g. `users/controller.rs`, `payments/controller.rs`, `waves/controller.rs`). Prefer generic user-facing messages and log detailed errors only on the server. |

### 2.7 Configuration & Hardening

| ID | Severity | Finding |
|----|----------|--------|
| **CONF-1** | **Medium** | **CORS allows any origin.** In `server.rs`, `Cors::default().allow_any_origin()` is used. Any website can send credentialed requests if the frontend uses credentials. **Fix:** In production, set `.allowed_origin()` (or a predicate) to the exact frontend origin(s). |
| **CONF-2** | **Low** | **Server binds to `0.0.0.0:8080`.** Service is reachable on all interfaces. Ensure deployment uses a reverse proxy and firewall; do not expose 8080 directly to the internet if not intended. |
| **CONF-3** | **Info** | **actix-limitation.** The app uses `RateLimiter::default()`; confirm it is configured (e.g. via `configure_rate_limiter`) so that a default limit is applied and Redis is used as intended. |

### 2.8 Session & Token Handling

| ID | Severity | Finding |
|----|----------|--------|
| **SESS-1** | **Low** | **Refresh token long-lived.** Refresh tokens are valid 30 days. If stolen, an attacker has a long window. Consider shorter lifetime plus rotation, or binding to session and invalidating on logout. |
| **SESS-2** | **Info** | **Google OAuth state.** The controller checks a session cookie value against the `state` parameter, which mitigates CSRF for the OAuth callback. |

---

## 3. Penetration Possibilities (Attack Scenarios)

### 3.1 Privilege Escalation / Admin Abuse

- **List permissions:** Any authenticated user can call `GET /auth/permissions` and enumerate all permission names, aiding further privilege abuse or social engineering.

### 3.2 Information Gathering (Unauthenticated)

- **User enumeration and data:** `GET /users?page=1&per_page=20` (and filters) returns user list with obfuscated email, username, realname, role. No auth required.
- **Cached exports:** `GET /export/cached` lists cached exports; `GET /export/cached/{language_tag}/{format}` downloads them. No auth required; an attacker can download full dictionary exports.

### 3.3 Resource Abuse

- **Assistant API:** Unauthenticated POST to `/assistant/chat` consumes OpenRouter (or similar) and server CPU; can be used for free inference or DoS against the external API.

### 3.4 Credential and Session Risk

- **Legacy passwords:** If the `users.password` table is leaked, MD5(ROT13(pass)) hashes are trivial to crack; an attacker can then log in as those users.
- **Error details:** Invalid refresh token or comment errors may reveal internal paths, types, or DB details, helping an attacker refine further attacks.

### 3.5 Bypassing Protections

- **CORS:** Malicious site can send credentialed requests to the API if the frontend uses cookies/credentials, depending on browser and cookie flags.

---

## 4. How to Fix (Prioritized)

### 4.1 High Priority

1. **AUTH-1 – Legacy MD5 passwords**  
   - Stop supporting MD5 in `verify_password()`.  
   - Force password reset or rehash on next login for any user with a 32-hex stored hash; then remove the MD5 branch.

2. **ACL-1 – Permissions endpoint**  
   - Add `#[protect(any("manage_roles"))]` (or the correct permission) to `get_permissions` in `auth/controller.rs`.  
   - Ensure OpenAPI matches.

3. **ACL-2 – List users**  
   - If admin-only: move `list_users` under the bearer scope and add `#[protect(any("manage_users"))]` (or appropriate permission).  
   - If public by design: document and consider removing or further obfuscating sensitive fields.

4. **ACL-3 – Cached exports**  
   - Either protect `list_cached_exports` and `download_cached_export` with the same bearer auth as `export_dictionary`, or explicitly document public access and any impact.

5. **ACL-4 – Assistant chat**  
   - Require authentication (e.g. wrap the assistant scope with `HttpAuthentication::bearer(crate::auth::validator)`).  
   - Optionally add per-user rate limiting.

### 4.2 Medium Priority

6. **CONF-1 – CORS**  
   - Replace `allow_any_origin()` with an explicit list or predicate (e.g. from env) for the frontend origin(s).

7. **INFO-1, INFO-2, INFO-3 – Error details**  
   - Remove `e.to_string()` and `format!("{:#?}", e)` from JSON/body responses.  
   - Return fixed, user-safe messages; log full errors with `log::error!` or similar.

8. **ACL-5 – block_user / assign_role**  
    - Add `#[protect(any("block_users"))]` and `#[protect(any("manage_roles"))]` respectively for consistency and defense in depth.

### 4.3 Lower Priority

9. **INP-1 – Redis keys**  
    - Normalize/sanitize IP and email/identifier before using in Redis keys (length limit, strip control characters).

10. **INP-2 – Content-Disposition**  
    - Sanitize or harden `filename` (no `"`, `\`, CR, LF) before putting it in the `Content-Disposition` header.

11. **AUTH-2 – Env panics**  
    - Load `JWT_SECRET`, `REFRESH_TOKEN_SECRET`, `FRONTEND_URL`, `GOOGLE_*` at startup and return a config error instead of `.expect()` in request path.

12. **SESS-1 – Refresh token lifetime**  
    - Consider shorter refresh token TTL and/or refresh token rotation and binding to session.

---

## 5. Positive Findings

- **SQL:** No raw concatenation of user input into SQL; parameterized queries used throughout.
- **Ownership checks:** Collections, flashcards, and subscription flows use `verify_collection_ownership` / equivalent and pass `claims.sub` or `user_id` correctly in the services checked.
- **Login and sensitive flows:** Login, password reset, and email confirmation are rate-limited; password reset response does not reveal email existence.
- **OAuth:** Google OAuth callback validates `state` against session cookie, reducing CSRF risk.
- **HTML:** User-supplied text (profile, collection name/description) is sanitized with Ammonia (no tags) before storage.

---

## 6. Summary Table

| Category        | Count |
|----------------|-------|
| High            | 2 (AUTH-1, plus ACL-1/2/3/4 as a group) |
| Medium          | 7     |
| Low             | 7     |
| Informational   | 4     |

**Recommended order of work:** Fix permissions ACL (ACL-1) and remove MD5 support (AUTH-1) first, then lock down list users, cached exports, and assistant (ACL-2, ACL-3, ACL-4), then CORS and error disclosure (CONF-1, INFO-*).
