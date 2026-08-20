# Social login (GitHub and Google)

Lensisku uses one OAuth pipeline for every identity provider. The Vue app never holds client secrets: it asks the API for an authorization URL, the user signs in at GitHub or Google, then the SPA callback posts the `code` back to the API.

Configured providers appear as buttons on login and signup (main site and Lingo). If a provider’s environment variables are missing, it is omitted from `GET /auth/oauth/providers` and its authorize/complete endpoints return **503**.

## Architecture

| Step | Endpoint | Role |
| --- | --- | --- |
| List | `GET /auth/oauth/providers` | `{ "providers": ["github", "google"] }` for whichever env sets are complete |
| Start | `GET /auth/oauth/{provider}/authorize?return_to=` | HMAC-signed `state` (nonce, ~10 minute expiry, provider, optional in-app `return_to`) and the IdP `authorize_url` |
| Callback page | Browser `GET /oauth/{provider}` | Locale-independent SPA route (not under `/en/…`) so the IdP can use a single exact redirect URI |
| Finish | `POST /auth/oauth/{provider}` `{ "code", "state" }` | Token exchange, profile, account link/create, `access_token` + `refresh_token` (same as password login) |

Redirect URI for each provider **must** be the frontend origin plus `/oauth/{provider}`:

- Local: `http://localhost:5173/oauth/github` and `http://localhost:5173/oauth/google`
- Production: `https://lensisku.lojban.org/oauth/github` and `https://lensisku.lojban.org/oauth/google`

Set `{PROVIDER}_REDIRECT_URL` on the **API** to that exact string. A mismatch with the value registered at the IdP causes `redirect_uri` errors.

Environment variables (all three required for that provider to be enabled):

| Provider | Variables |
| --- | --- |
| GitHub | `GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET`, `GITHUB_REDIRECT_URL` |
| Google | `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `GOOGLE_REDIRECT_URL` |

Never commit secrets. Put them in the server `.env` (or compose/host secrets), not in `frontend/.env`. There is no `VITE_GITHUB_CLIENT_ID` / `VITE_GOOGLE_CLIENT_ID`.

Account behaviour (all providers):

1. Existing `oauth_accounts` row for `(provider, provider_id)` → log in.
2. Else a **verified** email that already exists on a user → link `oauth_accounts` and log in. Unverified emails are **not** linked.
3. Else create a user (`oauth_signup`, confirmed so they can use the site). Username comes from GitHub `login` or the Google email local-part, with `-2`, `-3`, … on collision.

## GitHub (lojban org OAuth App)

GitHub **OAuth Apps** allow **one** Authorization callback URL. Create **two** apps under the lojban organization so local and production do not share a callback (and so the app stays if a personal maintainer leaves).

1. Sign in with an account that can manage [github.com/lojban](https://github.com/lojban).
2. Open **Organization settings → Developer settings → OAuth Apps → New Org OAuth App**:  
   [https://github.com/organizations/lojban/settings/applications/new](https://github.com/organizations/lojban/settings/applications/new)
3. Production app (example):
   - **Application name:** `Lensisku`
   - **Homepage URL:** `https://lensisku.lojban.org`
   - **Authorization callback URL:** `https://lensisku.lojban.org/oauth/github`
4. Local app (example):
   - **Application name:** `Lensisku (local)`
   - **Homepage URL:** `http://localhost:5173`
   - **Authorization callback URL:** `http://localhost:5173/oauth/github`
5. After create: copy **Client ID**. Click **Generate a new client secret** and copy it once.
6. On the API host (or local `.env`):

   ```bash
   GITHUB_CLIENT_ID=...
   GITHUB_CLIENT_SECRET=...
   GITHUB_REDIRECT_URL=http://localhost:5173/oauth/github
   ```

   Production uses the production callback URL and the production app’s id/secret.

7. Restart the API. Confirm `GET /auth/oauth/providers` includes `"github"`.
8. On login or signup, use **Continue with GitHub**. After GitHub, you should land back in the app signed in.

Scopes requested: `read:user` `user:email` (needed to read verified emails).

This is an **OAuth App**, not a GitHub App (no installation on repositories).

## Google Cloud (Web client)

A Google **Web application** OAuth client can list several authorized redirect URIs. Prefer **separate** clients for local and production so secrets stay isolated.

1. Open [Google Cloud Console](https://console.cloud.google.com/) and select (or create) a project used for Lensisku.
2. **APIs & Services → OAuth consent screen**:
   - User type: External (unless the project is Google Workspace–only).
   - App name, support email, developer contact.
   - Scopes: `openid`, `email`, `profile` (or “…/auth/userinfo.email” and “…/auth/userinfo.profile”).
   - For testing before verification, add your Google account under **Test users**.
3. **APIs & Services → Credentials → Create credentials → OAuth client ID**:
   - Application type: **Web application**.
   - Name: `Lensisku` or `Lensisku (local)`.
   - **Authorized JavaScript origins:** `https://lensisku.lojban.org` or `http://localhost:5173`.
   - **Authorized redirect URIs:** `https://lensisku.lojban.org/oauth/google` or `http://localhost:5173/oauth/google`.
4. Copy Client ID and Client Secret into the API environment:

   ```bash
   GOOGLE_CLIENT_ID=....apps.googleusercontent.com
   GOOGLE_CLIENT_SECRET=...
   GOOGLE_REDIRECT_URL=http://localhost:5173/oauth/google
   ```

5. Restart the API. Confirm `GET /auth/oauth/providers` includes `"google"`.
6. Click **Continue with Google** on login/signup.

Publishing the consent screen for production (beyond test users) may require Google’s verification if you use sensitive scopes; `email` / `profile` / `openid` are typically sufficient for this flow.

## Adding another provider later

1. Backend: extend `OAuthProvider` in [`src/auth/oauth.rs`](../src/auth/oauth.rs) (id string, `{PREFIX}_CLIENT_ID` / `_SECRET` / `_REDIRECT_URL`, authorize URL, token URL, scopes, profile fetch → `provider_id`, email, `email_verified`, suggested username).
2. Register the callback `https://<frontend>/oauth/<id>` at the IdP and in `{PREFIX}_REDIRECT_URL`.
3. Frontend: add icon + `oauth.continueWith…` i18n keys in [`frontend/src/components/SocialLoginButtons.vue`](../frontend/src/components/SocialLoginButtons.vue) and all locale JSON files. Buttons are driven by `/auth/oauth/providers`; unknown ids are ignored.
4. Document the console steps in this file.

No new signup route is required; `GET/POST /auth/oauth/{provider}` already generic.

## Troubleshooting

| Symptom | Likely cause |
| --- | --- |
| Button missing | Env vars unset or empty; API not restarted; `GET /auth/oauth/providers` does not list the provider |
| `redirect_uri` mismatch / 400 from GitHub or Google | `{PROVIDER}_REDIRECT_URL` ≠ IdP callback ≠ browser origin `/oauth/{provider}` (scheme, host, port, path, no trailing slash) |
| 503 `not_configured` | Incomplete `{PROVIDER}_*` set on the **API** process |
| Sign-in cancelled | User denied the consent screen (`error=access_denied`) |
| Collision / 409 | An existing account already has that email, but the IdP email is not verified, so the accounts are not linked. Sign in with password. |
| GitHub has no email | Need `user:email` scope; on GitHub keep at least one email visible/verified |
| Local works, production does not | Wrong OAuth app (GitHub) or wrong client (Google); production still pointing at `localhost` redirect |

Local frontend is `http://localhost:5173/` with API typically `http://localhost:8080` (`VITE_BASE_URL`). The OAuth callback is on the **frontend** origin, not on `:8080`.
