
# Console Deployment

This document explains how to deploy SIGNIA Console in real environments. It includes:
- local development
- static hosting deployment
- environment configuration
- security posture (API key handling)
- embedding as a widget
- CI/CD examples

Related docs:
- `docs/console/overview.md`
- `docs/console/interface-module.md`
- `docs/api/auth.md`
- `docs/api/openapi.yaml`

---

## 1) What you deploy

SIGNIA Console is a frontend application that talks to the SIGNIA API.

Deployment modes:
1. Standalone app
- A full UI hosted at a domain (static or server-rendered)

2. Embedded widget
- A smaller component embedded into an existing website

In both modes, the Console needs:
- an API base URL
- an auth mechanism (usually an API key)

---

## 2) Local development

### 2.1 Prerequisites
- Node.js (LTS recommended)
- pnpm or npm
- SIGNIA API running locally

### 2.2 Run the API
Example:
```bash
cargo run -p signia-api -- --bind 0.0.0.0:8787
```

Set an API key:
```bash
export SIGNIA_API_KEY="sk_signia_REDACTED"
```

### 2.3 Run the Console
Example (Vite/Next-style, adjust for your repo):
```bash
cd apps/console
pnpm install
pnpm dev
```

Open:
- `http://localhost:3000`

---

## 3) Configuration

### 3.1 Build-time configuration
The Console should support build-time environment variables:

- `SIGNIA_CONSOLE_DEFAULT_API_BASE_URL`
- `SIGNIA_CONSOLE_FEATURE_ONCHAIN` (true/false)
- `SIGNIA_CONSOLE_FEATURE_REMOTE_URLS` (true/false)
- `SIGNIA_CONSOLE_FEATURE_UPLOADS` (true/false)

Example:
```bash
SIGNIA_CONSOLE_DEFAULT_API_BASE_URL="https://api.signia.yourdomain.tld" \
SIGNIA_CONSOLE_FEATURE_ONCHAIN="true" \
pnpm build
```

### 3.2 Runtime configuration (recommended)
Static hosting often cannot change env after build. Use runtime config via:
- `config.json` fetched on startup
- or injected `<script>` config

Example `public/signia.config.json`:
```json
{
  "apiBaseUrl": "https://api.signia.yourdomain.tld",
  "features": {
    "onchain": true,
    "remoteUrls": false,
    "uploads": true
  }
}
```

The Console loads this at startup and uses it as defaults.

---

## 4) Auth and API key handling

### 4.1 Recommended posture
Do not embed a privileged API key in a public frontend.

Safer options:
1. Run a gateway/proxy
- The Console calls your gateway
- The gateway injects the real API key server-side

2. Use per-user keys
- Users bring their own key (stored locally)
- Best for self-hosted/internal deployments

3. Use JWT/OIDC
- Console obtains tokens from your IdP
- API validates tokens

### 4.2 Public vs private Console
- Public Console (marketing site): should not have privileged access.
  - disable uploads
  - allow only read-only endpoints or demo mode

- Private Console (internal tools): can store a key in browser storage, but:
  - use short-lived tokens if possible
  - protect via SSO
  - isolate by network (VPN)

### 4.3 Browser storage
If you store keys client-side:
- prefer session storage
- avoid localStorage unless necessary
- provide a “forget key” button

Never put keys in URLs.

---

## 5) Static hosting deployment

### 5.1 Build
```bash
cd apps/console
pnpm install
pnpm build
```

This produces:
- `dist/` (Vite) or `.next/` + `out/` (Next export) depending on framework

### 5.2 Host
Options:
- Cloudflare Pages
- Vercel
- Netlify
- S3 + CloudFront
- GitHub Pages (for static export)

### 5.3 Example: Cloudflare Pages (static)
- connect repo
- set build command: `pnpm -C apps/console build`
- set output directory: `apps/console/dist`
- set environment variables as needed

---

## 6) Deploy behind a reverse proxy (recommended)

If you need:
- consistent HTTPS
- path-based routing
- security headers

Put the Console behind Nginx/Caddy/Traefik.

Example Nginx snippet:

```nginx
server {
  listen 443 ssl;
  server_name console.signia.yourdomain.tld;

  location / {
    root /var/www/console;
    try_files $uri /index.html;
  }

  add_header X-Frame-Options "SAMEORIGIN" always;
  add_header X-Content-Type-Options "nosniff" always;
  add_header Referrer-Policy "no-referrer" always;
  add_header Content-Security-Policy "default-src 'self'; connect-src 'self' https://api.signia.yourdomain.tld; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline';" always;
}
```

Adjust CSP for your needs.

---

## 7) Embedding as a widget

### 7.1 Build as a library bundle
If the Console supports widget mode:
- export an entrypoint `SigniaConsoleWidget`
- build as an ES module

Example usage:

```html
<div id="signia-console"></div>
<script type="module">
  import { mountSigniaConsole } from "/signia-console-widget.js";
  mountSigniaConsole(document.getElementById("signia-console"), {
    apiBaseUrl: "https://api.signia.yourdomain.tld",
    features: { onchain: true }
  });
</script>
```

### 7.2 iframe embed
For isolation, embed in an iframe:
- host the Console at `console.signia...`
- embed:

```html
<iframe
  src="https://console.signia.yourdomain.tld/embed?mode=verify"
  width="100%"
  height="800"
  style="border:0"
></iframe>
```

Ensure `X-Frame-Options` and CSP allow this intentionally.

---

## 8) CORS and networking

The API must allow the Console origin.

Recommended:
- run Console and API under the same parent domain:
  - `console.signia.tld`
  - `api.signia.tld`

CORS config:
- allow `GET, POST, DELETE`
- allow headers: `Content-Type`, `X-API-Key`
- do not use `*` for `Access-Control-Allow-Origin` in production unless necessary

---

## 9) Production hardening checklist

- HTTPS enabled
- CSP configured
- remove privileged API keys from public builds
- uploads disabled for public deployment unless protected
- rate limits on API enabled
- logs redacted (no keys)
- error messages do not leak internal paths

---

## 10) CI/CD example

### 10.1 GitHub Actions build
Example job:
- install node
- build console
- upload artifact
- deploy

Pseudo steps:
```yaml
- uses: actions/setup-node@v4
  with:
    node-version: "20"
- run: corepack enable
- run: pnpm -C apps/console install
- run: pnpm -C apps/console build
```

### 10.2 Preview deployments
Use PR previews on:
- Vercel/Netlify/Cloudflare Pages

Ensure previews point to:
- staging API
- or a mocked API

---

## 11) Troubleshooting

### 11.1 401 errors
- missing or invalid API key
- baseUrl incorrect
- gateway not injecting key

### 11.2 CORS errors
- API origin not allowed
- missing `Access-Control-Allow-Headers: X-API-Key`

### 11.3 Upload failures
- max request size exceeded
- reverse proxy body size limit too low
- API input limits too strict

### 11.4 Rate limits
- observe `Retry-After`
- reduce polling frequency
- add exponential backoff

---

## 12) Related documents

- Console overview: `docs/console/overview.md`
- Interface module: `docs/console/interface-module.md`
- Auth: `docs/api/auth.md`
- OpenAPI: `docs/api/openapi.yaml`
