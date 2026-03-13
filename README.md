# GrindIt

A workout tracking app inspired by SugarWOD, built with Rust. Features WOD programming, workout logging, exercise library, leaderboards, and role-based access for coaches and athletes.

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
    mod.rs            # AuthUser, UserRole, get_me() server function
    oauth.rs          # Google OAuth handlers (login, callback, logout)
    session.rs        # Session helpers (get_current_user, require_auth, require_role)

  pages/
    home.rs           # Dashboard with stats, streak, leaderboard
    exercises/
      mod.rs          # ExercisesPage, server fns: list/create/update/delete, category constants
      exercise_card.rs  # Exercise card with inline edit, video embed toggle, delete
      exercise_form.rs  # Create exercise form (FAB-triggered)
    wod/
      mod.rs          # WodPage, server fns: get_week_dates, list_wods_for_date, CRUD actions
      week_calendar.rs  # Sticky weekly calendar (Sun–Sat), selects date to show WODs
      wod_card.rs     # Collapsible WOD card with inline coach edit/delete
      wod_form.rs     # Create WOD form
      wod_sections_panel.rs  # Lists sections for a WOD; coach add/delete section
      wod_section_card.rs    # Section card with movements list and "Log Result" button
      section_movements_panel.rs  # Movements per section; coach add/delete
    log_workout.rs    # Log workouts per section (WIP — form coming)
    history.rs        # Weekly calendar view of past workouts
    login.rs          # Google sign-in page
    profile.rs        # User profile, stats, sign out
    admin.rs          # User management (admin only)

  routes/
    health_check.rs   # GET /api/v1/health_check
    upload.rs         # POST /api/v1/upload/video

  components/
    _header.scss      # Top bar + bottom nav styles

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
| Auth | Google OAuth 2.0 + server-side sessions (tower-sessions) |
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
 └── section_logs         (one per section: finish time, rounds, rx/scaled)
      └── workout_exercises (one row per set per movement)
```

**Workout types** (both WOD and section level): `fortime`, `amrap`, `emom`, `tabata`, `strength`, `custom`

### Database Migrations

| # | Migration | Description |
|---|-----------|-------------|
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
| 10 | `create_section_logs_rework_workout_exercises` | `section_logs` table; `workout_exercises` becomes per-set linked to section_log |

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
