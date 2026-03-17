# GrindIt

A workout tracking app inspired by SugarWOD, built with Rust. Features WOD programming, workout logging, exercise library, history with edit/delete, and role-based access for coaches and athletes.

## Architecture

```
src/
  app.rs              # Root Leptos component, router, reactive bottom nav, auth gate
  main.rs             # Axum server setup, middleware, route mounting
  lib.rs              # Crate root, WASM hydration entry point
  configuration.rs    # YAML + env var config loading
  db.rs               # Database queries (SQLx, runtime-checked)
  storage.rs          # Storage backend abstraction (local / Cloudflare R2)
  telemetry.rs        # Tracing subscriber setup
  pwa.rs              # Service worker registration (WASM)
  voice.rs            # Client-side JS interop (file upload, theme toggle)

  auth/
    mod.rs            # AuthUser, UserRole, OtpResult, clean_error(), get_me()
    oauth.rs          # Google OAuth handlers (login, callback, logout)
    otp.rs            # SMS OTP send/verify via 2Factor.in (cryptographic OTP, atomic verify)
    password.rs       # Email/password login and registration server functions
    session.rs        # Session helpers (get_current_user, require_auth, require_role, auth_context)
    validation.rs     # Shared validators: validate_name, validate_email, validate_password, hash_password, default_role_for_new_user

  pages/
    home/
      mod.rs          # Dashboard with stats, streak, leaderboard
      leaderboard.rs  # Leaderboard component
    exercises/
      mod.rs          # ExercisesPage, server fns: list/create/update/delete
      exercise_card.rs  # Exercise card with inline edit, video embed toggle, delete
      exercise_form.rs  # Create exercise form (FAB-triggered)
      server_fns.rs   # Server functions for exercise CRUD
      helpers.rs      # Category/type helpers
    wod/
      mod.rs          # WodPage, server fns: list_wods_for_date, CRUD actions
      week_calendar.rs  # Sticky weekly calendar (Sun–Sat), computes dates client-side
      wod_card.rs     # Collapsible WOD card with inline coach edit/delete
      wod_form.rs     # Create WOD form
      wod_sections_panel.rs  # Lists sections for a WOD; coach add/delete section
      wod_section_card.rs    # Section card with movements list and "Log Result" button
      section_movements_panel.rs  # Movements per section; coach add/delete
      server_fns.rs   # Server functions for WOD CRUD
      helpers.rs      # WOD type/phase helpers
    log_workout/
      mod.rs          # LogWorkoutPage, tab switcher (WOD / Custom), query param routing
      wod_score_form.rs     # Per-section score entry form (new submissions only)
      section_score_card.rs # Individual section score card (Rx/skip toggles, time/rounds/weight)
      custom_log.rs   # Custom (non-WOD) workout logger
      exercise_entry_card.rs  # Exercise card for custom log
      set_row.rs      # Set input row (reps, weight, duration, notes)
      server_fns.rs   # Server functions: save WOD scores and custom logs
    history/
      mod.rs                   # HistoryPage, server fns: fetch/delete/update entries, reactive feed
      history_card.rs          # HistoryCard: layout, custom workout exercise display, delete button
      editable_section_row.rs  # Inline section-score editor (tap to edit time/rounds/weight/Rx)
      editable_movement_row.rs # Inline per-set movement editor (per-set or flat reps/weight)
    login/
      mod.rs          # Phone OTP + Email/Password tabs, Google OAuth button, toast errors
    profile/
      mod.rs          # User profile (name, email, phone, gender), set/update password, stats, sign out
    admin/
      mod.rs          # User management (admin only)
      user_row.rs     # User row with role selector

  routes/
    health_check.rs   # GET /api/v1/health_check
    upload.rs         # POST /api/v1/upload/video

  components/
    mod.rs            # Component re-exports
    delete_modal.rs   # Reusable delete confirmation modal
    header.rs         # Top app bar + bottom nav
    multi_select.rs   # Multi-select dropdown
    single_select.rs  # Single-select dropdown
    video_upload.rs   # Video file upload with progress

style/
  main.scss           # Entry point — imports all modules
  _themes.scss        # CSS custom properties: dark (Ember) and light (Himalayan Snow)
  _reset.scss         # Base reset and global defaults
  _toast.scss         # Global toast notification: slide-in from right, auto-dismiss (4.75s)

configuration/
  base.yaml           # Shared defaults
  local.yaml          # Local dev overrides
  production.yaml     # Production overrides (SSL, R2)

migrations/           # PostgreSQL migrations (auto-run on startup)
public/               # PWA manifest, service worker, icons
scripts/
  init_db.sh          # Local DB setup script
  pre-commit          # Pre-commit hook (fmt + clippy)
  setup-hooks.sh      # Install git hooks
```

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | Leptos 0.8 (SSR + WASM hydration) |
| Server | Axum 0.8 |
| Database | PostgreSQL via SQLx |
| Auth | Google OAuth 2.0 + Email/Password + Phone OTP (2Factor.in) + server-side sessions (tower-sessions) |
| Storage | Local filesystem (dev) / Cloudflare R2 (prod) |
| Styling | SCSS |
| PWA | manifest.json + service worker |
| Build | cargo-leptos |

### Exercise Library

**Categories** (17 total): Conditioning, Gymnastics, Weightlifting, Powerlifting, Cardio, Bodybuilding, Strongman, Plyometrics, Calisthenics, Mobility, Yoga, Meditation, Breathing, Chanting, Sports, Warm Up, Cool Down

Each exercise stores: `name`, `category`, `movement_type` (optional), `description` (optional), `demo_video_url` (optional — YouTube, Vimeo, or direct upload via R2). Video URLs are normalised to embed form (e.g. `youtube.com/watch?v=` → `youtube.com/embed/`).

**Access:**
- Anyone can browse
- Any authenticated user can create exercises
- Any authenticated user can edit/delete exercises

### WOD Data Model

WODs follow a three-level hierarchy:

```
wod
 └── wod_sections          (warmup / strength / conditioning / cooldown / optional / personal)
      └── wod_movements     (exercise + rep_scheme + male/female weights)
```

Logging mirrors this structure:

```
workout_logs              (one per athlete per day, linked to a wod)
 └── section_logs         (one per section: finish time, rounds, rx/scaled, score_value, weight_kg)
      └── movement_logs   (per-movement results: actual reps, sets, weight within a section)
           └── movement_log_sets  (per-set rows: set_number, reps, weight_kg, distance_meters, calories)

workout_logs              (custom / non-WOD)
 └── workout_exercises     (linked directly via workout_log_id)
```

**Workout types** (both WOD and section level): `fortime`, `amrap`, `emom`, `tabata`, `strength`, `custom`

**Exercise scoring types**: `weight_and_reps`, `reps_only`, `distance`, `calories`, `time`

### Rep Scheme Parsing

When a coach programs a movement's rep scheme (`wod_movements.rep_scheme`), the log form parses it to pre-populate per-set input rows. The scheme format depends on the exercise's scoring type:

| Scoring type | Example scheme | Parsed as |
|---|---|---|
| `weight_and_reps` / `reps_only` | `21-15-9` | 3 sets: 21, 15, 9 reps |
| `weight_and_reps` / `reps_only` | `5x5` | 5 sets of 5 reps |
| `weight_and_reps` / `reps_only` | `10` | single set, 10 reps (no per-set rows) |
| `distance` | `500m-500m-400m` | 3 sets: 500m, 500m, 400m |
| `distance` | `4x500m` | 4 sets of 500m |
| `calories` | `12-10-8` | 3 sets: 12, 10, 8 cal targets |
| `calories` | `4x12cal` | 4 sets of 12 cal |

Rules:
- Per-set rows appear only when the scheme produces **2 or more sets**. Single-value schemes fall back to flat inputs.
- Distance values strip any trailing alphabetic unit (`m`, `km`) before parsing.
- Calorie values strip any trailing alphabetic unit (`cal`) before parsing.
- Rep scheme strings not matching any pattern produce no pre-population (empty flat inputs).

### WOD Ownership

Only the coach who created a WOD can edit or delete it (and its sections/movements). Admins can edit any WOD. The `wods.created_by` column stores the creator's user ID; all mutation server functions enforce this check server-side.

### Database Migrations

| # | File | Description |
|---|------|-------------|
| 0 | `create_exercises_table` | Exercise library |
| 1 | `create_workout_logs_table` | Top-level workout log |
| 2 | `create_workout_exercises_table` | Per-exercise log rows |
| 3 | `create_users_table` | Users with Google OAuth identity |
| 4 | `add_user_id_to_tables` | Link logs and exercises to users |
| 5 | `add_rx_column` | Rx/scaled flag on workout log |
| 6 | `create_wods_table` | WODs + flat movements (initial) |
| 7 | `add_enums_gender_wod_sections` | `wod_phase`, `section_type`, `gender` enums; `wod_sections` table |
| 8 | `rework_wod_movements` | Movements linked to sections; male/female weights; rep_scheme text |
| 9 | `rework_workout_logs` | Link `workout_logs` to WODs; drop legacy name/type/duration columns |
| 10 | `create_section_logs_rework_workout_exercises` | `section_logs` table; `workout_exercises` per-set linked to section_log |
| 11 | `add_score_columns` | `weight_kg`, `score_value` on `section_logs`; unique constraint + leaderboard index |
| 12 | `add_workout_log_id_back_to_exercises` | `workout_log_id` on `workout_exercises` for custom (non-WOD) logs; parent check constraint |
| 13 | `add_password_auth` | `google_id` made nullable; `password_hash` column added (email/password auth) |
| 14 | `add_updated_at_to_workout_logs` | `updated_at TIMESTAMPTZ` on `workout_logs` (required for update queries) |
| 15 | `create_movement_logs_table` | Per-movement results within a section log (reps, sets, weight) |
| 16 | `add_phone_auth` | `phone` column on users; `otp_codes` table for SMS login |
| 17 | `add_gender_to_users` | `gender` column on users (male/female/null) for weight prescriptions |
| 18 | `create_movement_log_sets_table` | Per-set detail rows for movement logs (set_number, reps, weight_kg) |
| 19 | `add_distance_calories_to_movement_log_sets` | `distance_meters` and `calories` columns on movement_log_sets |

### Roles

- **Anonymous** -- browse WOD and Exercises pages (read-only)
- **Athlete** (default on signup) -- log workouts, view exercises & WODs, add exercises
- **Coach** (promoted by admin) -- create/edit/delete WODs and movements
- **Admin** (first user to sign up) -- manage user roles

## Local Development

### Prerequisites

- Rust nightly: `rustup default nightly`
- WASM target: `rustup target add wasm32-unknown-unknown`
- cargo-leptos: `cargo install cargo-leptos`
- Dart Sass: [install guide](https://sass-lang.com/install/)
- PostgreSQL 15+
- sqlx-cli: `cargo install sqlx-cli --no-default-features --features rustls,postgres`

### Database Setup

```bash
# Start Postgres and run migrations (uses Docker)
./scripts/init_db.sh

# Or if Postgres is already running:
SKIP_DOCKER=true ./scripts/init_db.sh
```

### SMS OTP Setup (optional)

Phone OTP login uses [2Factor.in](https://2factor.in/). Without SMS config, OTP codes are logged to the console (dev mode).

1. Create a 2Factor.in account and get your API key
2. Create an OTP template (e.g. `GrndItOTP`) in the 2Factor.in dashboard
3. Add to `.env`:

```env
APP_SMS__API_KEY=your-2factor-api-key
```

### Google OAuth Setup

1. Go to [Google Cloud Console](https://console.cloud.google.com/) > APIs & Services > Credentials
2. Create an OAuth 2.0 Client ID (Web application)
3. Add `http://localhost:3000/auth/google/callback` as an authorized redirect URI
4. Copy the client ID and secret into `.env`:

```env
APP_OAUTH__GOOGLE_CLIENT_ID=your-client-id
APP_OAUTH__GOOGLE_CLIENT_SECRET=your-client-secret
```

### Git Hooks

```bash
./scripts/setup-hooks.sh
```

This installs a pre-commit hook that runs `cargo fmt --check` and `cargo clippy` (same checks as CI).

### Run

```bash
cargo leptos watch
```

App runs at `http://localhost:3000`. The first user to sign in gets the Admin role.

### Testing on iPhone with ngrok

ngrok tunnels your local server to a public HTTPS URL, which is required for:
- Google OAuth (callback URL must be HTTPS)
- PWA install prompts (service workers require HTTPS)
- iOS Safari testing with safe-area insets

**Step 1 — Install ngrok**

```bash
brew install ngrok
```

Or download from [ngrok.com](https://ngrok.com/download) and authenticate:

```bash
ngrok config add-authtoken <your-token>
```

**Step 2 — Start the app**

```bash
cargo leptos watch
```

**Step 3 — Start ngrok in a separate terminal**

```bash
ngrok http 3000
```

ngrok will print a URL like `https://abc123.ngrok-free.app`. Use this as your base URL on the phone.

**Step 4 — Add the ngrok URL to Google OAuth**

In [Google Cloud Console](https://console.cloud.google.com/) > APIs & Services > Credentials > your OAuth client:

- Add `https://abc123.ngrok-free.app/auth/google/callback` to **Authorized redirect URIs**

Also set in `.env` (so the server sends the right callback URL):

```env
APP_OAUTH__REDIRECT_URL=https://abc123.ngrok-free.app/auth/google/callback
```

Restart `cargo leptos watch` after changing `.env`.

**Step 5 — Open on iPhone**

Navigate to `https://abc123.ngrok-free.app` in Safari on your iPhone. To test as a PWA:

1. Tap the Share button → **Add to Home Screen**
2. Launch from the home screen icon — it runs in standalone mode (no browser chrome)

> **Note:** ngrok free tier URLs change on every restart. Update the OAuth redirect URI whenever you get a new URL. Consider [ngrok static domains](https://ngrok.com/docs/network-edge/domains/) (free tier gives one) to keep a stable URL.

## Production Deployment

### Infrastructure

- **Compute**: DigitalOcean App Platform (Docker)
- **Database**: Neon PostgreSQL
- **Storage**: Cloudflare R2 (S3-compatible)

### Environment Variables

```env
# App
APP_ENVIRONMENT=production
APP_APPLICATION__HOST=0.0.0.0
APP_APPLICATION__PORT=3000
LEPTOS_SITE_ADDR=0.0.0.0:3000

# Database (Neon)
APP_DATABASE__HOST=<neon-host>.neon.tech
APP_DATABASE__PORT=5432
APP_DATABASE__USERNAME=<neon-user>
APP_DATABASE__PASSWORD=<neon-password>
APP_DATABASE__DATABASE_NAME=gritwit
APP_DATABASE__REQUIRE_SSL=true
APP_DATABASE__CHANNEL_BINDING=false

# OAuth (Google)
APP_OAUTH__GOOGLE_CLIENT_ID=<google-client-id>
APP_OAUTH__GOOGLE_CLIENT_SECRET=<google-client-secret>
APP_OAUTH__REDIRECT_URL=https://<production-domain>/auth/google/callback

# SMS OTP (2Factor.in)
APP_SMS__API_KEY=<2factor-api-key>

# Storage (Cloudflare R2)
APP_STORAGE__BACKEND=r2
APP_STORAGE__R2_ACCOUNT_ID=<cloudflare-account-id>
APP_STORAGE__R2_ACCESS_KEY=<r2-access-key>
APP_STORAGE__R2_SECRET_KEY=<r2-secret-key>
APP_STORAGE__R2_BUCKET=gritwit
APP_STORAGE__R2_PUBLIC_URL=https://pub-<hash>.r2.dev
```

### Docker Build

```bash
docker build -t gritwit .
docker run -p 3000:3000 --env-file .env gritwit
```

Migrations run automatically on startup. No manual migration step needed.

### CI/CD

GitHub Actions (`.github/workflows/`):
- **general.yml** -- runs `cargo fmt`, `cargo clippy`, and `cargo test` on every push/PR
- **audit.yml** -- daily `cargo deny` security audit on dependency changes

## Production Hardening Checklist

### Done

- [x] Google OAuth with role-based access (Athlete/Coach/Admin)
- [x] Email/password authentication (register + login)
- [x] Phone OTP authentication via 2Factor.in (cryptographic OTP generation, atomic verification, rate limiting)
- [x] Server-side sessions in PostgreSQL
- [x] Environment-aware session cookies (Secure flag in production)
- [x] Cloudflare R2 storage backend for video uploads
- [x] SSL required for production database connections
- [x] Multi-stage Docker build with minimal runtime image
- [x] Health check endpoint (`/api/v1/health_check`)
- [x] Request ID tracing (x-request-id propagation)
- [x] CI pipeline (fmt, clippy, tests, security audit)
- [x] Pre-commit hooks (fmt + clippy)
- [x] Public pages (WOD, Exercises) accessible without sign-in
- [x] PWA install banner (Android/desktop native prompt, iOS instructions)
- [x] Upload validation (auth guard, magic bytes, extension allowlist, 100 MB body limit)
- [x] Loading indicators on all server actions (spinners + disabled buttons)
- [x] WOD programming: weekly calendar, sections, movements, coach inline edit/delete
- [x] Reactive bottom nav active state (client-side navigation aware)
- [x] Workout history: per-entry edit (re-opens log form) and delete (modal confirm)
- [x] Global toast notifications: top-right, slide-in animation, auto-dismiss (4.75s)
- [x] Weekly calendar computes dates client-side (no server round-trip on date change)
- [x] User profile page: edit name/email/phone/gender, set/update password with confirmation
- [x] Per-movement logging within WOD sections (actual reps, sets, weight per movement)
- [x] Per-set logging: multi-set rep schemes (`21-15-9`, `5x5`), distance intervals (`500m-500m`), calorie intervals (`12-10-8cal`) produce individual input rows
- [x] Gender-aware weight prescriptions (male/female weights on WOD movements)
- [x] WOD ownership guard: only the creator can edit/delete a WOD (admins bypass)
- [x] Inline history editing: WOD section scores and per-set movement results editable in-place on the history page (no redirect)

### Pending

- [ ] **Database backups** -- configure Neon's point-in-time recovery or periodic pg_dump
- [ ] **Rate limiting** -- add `tower_governor` or similar middleware to prevent abuse on auth and upload endpoints
- [ ] **CORS policy** -- restrict allowed origins in production
- [ ] **CSRF protection** -- add token validation for state-mutating server functions
- [ ] **Custom domain** -- move R2 public URL from r2.dev subdomain to a custom domain for caching + no rate limits
- [ ] **Monitoring & alerting** -- integrate with Sentry, Datadog, or similar for error tracking
- [ ] **Log aggregation** -- ship structured logs to a centralized service
- [ ] **Session cleanup** -- periodic job to purge expired sessions from PostgreSQL
- [ ] **Load testing** -- benchmark under expected traffic with `oha` or `k6`
- [ ] **Content Security Policy** -- add CSP headers to prevent XSS
- [ ] **HTTP security headers** -- HSTS, X-Frame-Options, X-Content-Type-Options
- [ ] **Graceful shutdown** -- handle SIGTERM for zero-downtime deploys
- [ ] **Database connection pooling** -- tune pool size for production workload
- [ ] **CDN** -- put Cloudflare or similar in front of the app for static asset caching
