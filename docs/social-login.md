# Social login (GitHub and Google)

Lensisku uses one OAuth pipeline for every identity provider. The Vue app never holds client secrets: it asks the API for an authorization URL, the user signs in at GitHub or Google, then the SPA callback posts the `code` back to the API.

Configured providers appear as buttons on login and signup (main site and Lingo). If a provider’s environment variables are missing or empty, it is omitted from `GET /auth/oauth/providers` and its authorize/complete endpoints return **503**.

## Architecture


| Step          | Endpoint                                            | Role                                                                                                              |
| ------------- | --------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| List          | `GET /auth/oauth/providers`                         | `{ "providers": ["github", "google"] }` for whichever env sets are complete                                       |
| Start         | `GET /auth/oauth/{provider}/authorize?return_to=`   | HMAC-signed `state` (nonce, ~10 minute expiry, provider, optional in-app `return_to`) and the IdP `authorize_url` |
| Callback page | Browser `GET /oauth/{provider}`                     | Locale-independent SPA route (not under `/en/…`) so the IdP can use a single exact redirect URI                   |
| Finish        | `POST /auth/oauth/{provider}` `{ "code", "state" }` | Token exchange, profile, account link/create, `access_token` + `refresh_token` (same as password login)           |


Redirect URI for each provider **must** be the frontend origin plus `/oauth/{provider}`:


| Environment                            | Frontend origin                   | Example GitHub redirect                        |
| -------------------------------------- | --------------------------------- | ---------------------------------------------- |
| Local Vite                             | `http://localhost:5173`           | `http://localhost:5173/oauth/github`           |
| Hosted **dev** (`web-dev`, port 20390) | `https://lensisku-dev.lojban.org` | `https://lensisku-dev.lojban.org/oauth/github` |
| Hosted **prod** (`web`, port 20380)    | `https://lensisku.lojban.org`     | `https://lensisku.lojban.org/oauth/github`     |


`{PROVIDER}_REDIRECT_URL` on the **API** must equal that exact string (and match a Redirect URL registered at GitHub/Google). There is no `VITE_GITHUB_CLIENT_ID` / `VITE_GOOGLE_CLIENT_ID`.

Runtime env names (inside the container / local `.env`):


| Provider | Variables                                                         |
| -------- | ----------------------------------------------------------------- |
| GitHub   | `GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET`, `GITHUB_REDIRECT_URL` |
| Google   | `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `GOOGLE_REDIRECT_URL` |


Account behaviour (all providers):

1. Existing `oauth_accounts` row for `(provider, provider_id)` → log in.
2. Else the IdP email already exists on a user (password signup or any other provider) → link `oauth_accounts` and log in (same as social signup when that email is already registered).
3. Else create a user (`oauth_signup`, confirmed so they can use the site). Username comes from GitHub `login` or the Google email local-part, with `-2`, `-3`, … on username collision.



## Where to store secrets (dev and prod containers)

Hosted Lensisku is built with **LBCS** from `lensisku-containers`.


| Piece                                 | Role                                                                                                                                                                                             |
| ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `lensisku-containers/secrets` | Gitignored `KEY=value` file on the deploy host. LBCS feeds these names into ERB when rendering Dockerfiles. **Keys must match the ERB variable names** (snake_case), not the Docker `ENV` names. |
| `containers/web/Dockerfile.erb`       | **Prod** image (`bundle=lensisku`, `https://lensisku.lojban.org`). Bakes `ENV GITHUB_`* / `GOOGLE_*` from `prod_*` secrets. Redirect URLs are hardcoded for prod.                                |
| `containers/web-dev/Dockerfile.erb`   | **Dev** image (`bundle=lensisku-dev`, `https://lensisku-dev.lojban.org`). Same with `dev_`* secrets and hardcoded `lensisku-dev` redirect URLs.                                                  |


Same pattern as existing secrets (`prod_jwt_secret`, `dev_openrouter_api_key`, `gleki_smtp_pass`, …). Putting `OPENROUTER_API_KEY=…` in `secrets` does **not** work — the ERB name must be `dev_openrouter_api_key` / `prod_openrouter_api_key`.

### Keys to add in `secrets`

```bash
# Prod (containers/web)
prod_github_client_id=
prod_github_client_secret=
prod_google_client_id=
prod_google_client_secret=

# Dev (containers/web-dev)
dev_github_client_id=
dev_github_client_secret=
dev_google_client_id=
dev_google_client_secret=
```

Fill in the Client ID / Client Secret from the GitHub OAuth App and Google Cloud Web client. Leave a value empty only if you intentionally disable that provider for that environment (the build still requires the **variable to exist**; an empty value is fine and hides the button at runtime).

Redirect URLs are set in the Dockerfiles (not in `secrets`):

- Prod: `https://lensisku.lojban.org/oauth/github` and `…/oauth/google`
- Dev: `https://lensisku-dev.lojban.org/oauth/github` and `…/oauth/google`



### Apply after editing `secrets` or `Dockerfile.erb`

Rebuild and restart the affected web container(s) with the usual LBCS flow (e.g. rebuild image for `web` / `web-dev`, then run it). Confirm:

```bash
curl -sS https://lensisku.lojban.org/api/auth/oauth/providers
curl -sS https://lensisku-dev.lojban.org/api/auth/oauth/providers
```



### Local developer machine (not LBCS)

For `make back` / Vite on the laptop, put the runtime names in the project root `.env` (see `[.env.example](../.env.example)`):

```bash
GITHUB_CLIENT_ID=...
GITHUB_CLIENT_SECRET=...
GITHUB_REDIRECT_URL=http://localhost:5173/oauth/github
GOOGLE_CLIENT_ID=...
GOOGLE_CLIENT_SECRET=...
GOOGLE_REDIRECT_URL=http://localhost:5173/oauth/google
```

Never put secrets in `frontend/.env`.

## GitHub (lojban org OAuth App)

Use a GitHub **OAuth App** (not a GitHub App). Org ownership under [lojban](https://github.com/lojban) keeps the app if a personal maintainer leaves.

In the GitHub UI the field is usually labelled **Redirect URL** / **Redirect URI** (or **Callback URL**). Older GitHub docs call the same field **Authorization callback URL**. That value must match `GITHUB_REDIRECT_URL` for the environment.

GitHub allows up to **10** redirect URIs on one OAuth App. You can register localhost, `lensisku-dev`, and production on one app, or use separate apps / secrets per environment.

### Why “redirect_uri is not supported” on lensisku-dev

If the OAuth App only lists:

`https://lensisku.lojban.org/oauth/github`

then login on **prod** works, but **dev** fails: the API on `web-dev` sends

`redirect_uri=https://lensisku-dev.lojban.org/oauth/github`

and GitHub rejects any URI that is not registered.

**Do not rely on wildcard matching to cover both hosts.** GitHub’s “wildcard matching” on a redirect URI only allows *subdomains or extra path segments under that same registered host* (for example, under `lensisku.lojban.org`).  
`lensisku-dev.lojban.org` is a **sibling** hostname under `lojban.org`, not a subdomain of `lensisku.lojban.org`, so a wildcard on the prod URI will **not** authorize the dev callback.

**Fix:** on the same OAuth App, use **Add redirect URI** and add the exact string:

`https://lensisku-dev.lojban.org/oauth/github`

(and localhost if you need it). Leave wildcard matching **off** for each URI unless you have a deliberate multi-tenant subdomain design.

1. Sign in with an account that can manage [github.com/lojban](https://github.com/lojban).
2. Open **Organization settings → Developer settings → OAuth Apps → New Org OAuth App**:
  [https://github.com/organizations/lojban/settings/applications/new](https://github.com/organizations/lojban/settings/applications/new)
3. Fill in:
  - **Application name:** `Lensisku`
  - **Homepage URL:** `https://lensisku.lojban.org`
  - **Redirect URL:** `https://lensisku.lojban.org/oauth/github`  
  Then **Add redirect URI** for each other environment you need (exact match, no wildcards):  
  `https://lensisku-dev.lojban.org/oauth/github`  
  `http://localhost:5173/oauth/github`
4. For each redirect URI, leave **wildcard matching** **off** (exact match only). Wildcards do not replace listing `lensisku-dev` separately.
5. Copy **Client ID**; **Generate a new client secret** and copy it once.
6. Put the id/secret into `lensisku-containers/secrets` as `prod_github_*` and/or `dev_github_*` (see above). For laptop-only testing, use `GITHUB_*` in `.env`.
7. Rebuild/restart the API container (or restart local `make back`). Confirm `/api/auth/oauth/providers` includes `"github"`.
8. On login or signup, use **Continue with GitHub**.

Scopes requested: `read:user` `user:email`.

## Google Cloud (Web client)

A Google **Web application** OAuth client can list several authorized redirect URIs. Prefer separate clients for prod vs dev if you want isolated secrets; otherwise one client with multiple URIs is fine.

1. Open [Google Cloud Console](https://console.cloud.google.com/) and select (or create) a project used for Lensisku.
2. **APIs & Services → OAuth consent screen**:
  - User type: External (unless Google Workspace–only).
  - App name, support email, developer contact.
  - Scopes: `openid`, `email`, `profile`.
  - For testing before verification, add **Test users**.
3. **Credentials → Create credentials → OAuth client ID**:
  - Application type: **Web application**.
  - **Authorized JavaScript origins:**  
  `https://lensisku.lojban.org`, `https://lensisku-dev.lojban.org`, and/or `http://localhost:5173`.
  - **Authorized redirect URIs:**  
  `https://lensisku.lojban.org/oauth/google`,  
  `https://lensisku-dev.lojban.org/oauth/google`,  
  `http://localhost:5173/oauth/google`.
4. Copy Client ID and Client Secret into `secrets` as `prod_google_*` / `dev_google_*` (or into local `.env` as `GOOGLE_*`).
5. Rebuild/restart; confirm `/api/auth/oauth/providers` includes `"google"`.

Publishing the consent screen for production (beyond test users) may require Google verification; `email` / `profile` / `openid` are typically enough for this flow.

## Adding another provider later

1. Backend: extend `OAuthProvider` in `[src/auth/oauth.rs](../src/auth/oauth.rs)`.
2. Register `https://<frontend>/oauth/<id>` at the IdP; set `{PREFIX}_REDIRECT_URL` (hardcoded in `Dockerfile.erb` for hosted envs).
3. Add `prod_<provider>_client_id` / `_secret` and `dev_…` to `secrets`, plus `ENV` lines in both web Dockerfiles.
4. Frontend: icon + i18n in `SocialLoginButtons.vue` and locale JSON files.



## Troubleshooting


| Symptom                                           | Likely cause                                                                                        |
| ------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| Button missing                                    | Env empty/unset; container not rebuilt after `secrets` change; `GET /auth/oauth/providers` empty    |
| ERB `undefined local variable …_github_client_id` | Key missing from `secrets`, or wrong name (must be `prod_github_client_id`, not `GITHUB_CLIENT_ID`) |
| `redirect_uri` mismatch / 400 / “not supported”   | `{PROVIDER}_REDIRECT_URL` ≠ Redirect URL at IdP ≠ browser origin `/oauth/{provider}`                |
| 503 `not_configured`                              | Incomplete `{PROVIDER}_*` on the **API** process                                                    |
| Works on prod, not on lensisku-dev                | GitHub/Google list only the **prod** redirect URI; add exact `https://lensisku-dev.lojban.org/oauth/{provider}`. Wildcard on the prod URI does **not** cover `lensisku-dev` (sibling host). Also check empty `dev_*` secrets / wrong keys. |
| Sign-in cancelled                                 | User denied consent (`error=access_denied`)                                                         |
| GitHub has no email                               | Need `user:email`; keep a verified email on the GitHub account                                      |


Local Vite frontend is `http://localhost:5173/` with API typically `http://localhost:8080` (`VITE_BASE_URL`). Hosted OAuth callbacks are on the **frontend** host (`lensisku` / `lensisku-dev`), not on a separate API port.