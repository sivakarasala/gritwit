# GrindIt: Learn Rust + Leptos by Building a Real Fitness Tracker

## Context
Create a **separate tutorial repository** (`gritwit-book`) that teaches Rust + Leptos + DSA + System Design by progressively building the GrindIt fitness tracker from scratch. The current GrindIt repo serves as the "finished product" reference implementation.

**Triple goal:** Production Rust/Leptos skills + DSA fluency + system design interview readiness.

**Pedagogy: Feature-Driven + Spotlight + Reps**
- Each chapter builds ONE complete feature end-to-end (DB → server → UI)
- Each chapter has ONE **spotlight** Rust concept taught in depth
- Supporting concepts are introduced lightly with 💡 "Spotlighted in Ch X" forward references
- **Rust Gym** drills after each feature: 2-3 isolated exercises on the spotlight concept
- **DSA in Context** boxes link real code to interview patterns
- **System Design Corner** sections frame architecture decisions as interview talking points
- **"Coming from JS/Python/Go?"** comparison boxes for every spotlight concept

---

## Decisions (Confirmed)

| Decision | Choice |
|----------|--------|
| **Format** | Separate tutorial repo (`gritwit-book`) with mdBook + `code/chXX/` compilable snapshots |
| **Pedagogy** | Feature-driven + Spotlight + Reps |
| **Scope** | Full 26 chapters: Part 0 (4 beginner-only) + 18 feature + 1 design reflection + 3 capstone |
| **Exercise style** | Instructions + progressive hints (collapsed) + full solution |
| **Language comparisons** | Experienced track: "Coming from JS/Python/Go?" boxes. Beginner track: "Programming Concept" explainer boxes |
| **DSA** | ~18 organic patterns woven in + 8 harder patterns in capstone |
| **System design** | Corner sections in chapters + dedicated capstone deep dive |
| **REST API** | Chapter 16 builds full REST API layer alongside server functions |
| **Two tracks** | Beginner (Level A: absolute zero + Part 0) and Experienced (skip Part 0, concise explanations) |
| **Design philosophy** | Ousterhout's *A Philosophy of Software Design* woven as "Design Insight" boxes + Ch 18.5 reflection chapter |
| **Versioning** | Hybrid: pinned core (book-v1 tag) + living evolution log (CI-powered drift detection) |

---

## Repository Structure

```
gritwit-book/
├── book.toml
├── src/
│   ├── SUMMARY.md
│   ├── introduction.md
│   │
│   ├── part-0-foundations/              # BEGINNER TRACK ONLY
│   │   ├── ch00-1-your-workshop.md      # Terminal, files, editors
│   │   ├── ch00-2-first-program.md      # What is code, cargo, hello world
│   │   ├── ch00-3-thinking.md           # Logic, debugging, problem solving
│   │   └── ch00-4-building-blocks.md    # Variables, functions, loops (Rust)
│   │
│   ├── beginner/                        # Beginner versions of Ch 1-18
│   │   ├── ch01-hello-grindit.md        # Thorough explanations, more screenshots
│   │   ├── ch02-exercise-library.md     # "Programming Concept" boxes
│   │   ├── ...                          # Simpler Rust Gym drills
│   │   └── ch18-ci-cd.md               # Step-by-step with why
│   │
│   ├── experienced/                     # Experienced versions of Ch 1-18
│   │   ├── ch01-hello-grindit.md        # Concise, "Coming from JS?" boxes
│   │   ├── ch02-exercise-library.md     # Interview-adjacent Rust Gym
│   │   ├── ...                          # Assumes programming knowledge
│   │   └── ch18-ci-cd.md
│   │
│   ├── ch18-5-design-reflection.md       # Shared — Ousterhout's philosophy applied
│   │
│   ├── capstone/                        # Shared by both tracks
│   │   ├── ch19-coding-challenges.md
│   │   ├── ch20-system-design.md
│   │   └── ch21-mock-interviews.md
│   │
│   └── evolution/                       # LIVING — auto-synced with GrindIt main
│       ├── README.md                    # Index of evolution entries
│       └── YYYY-MM-description.md       # Auto-scaffolded by CI when drift detected
│
├── code/
│   ├── ch00/ through ch18/              # Compilable project snapshots
│   └── capstone/                        # Standalone DSA exercises
└── README.md
```

**Content sharing:** ~60% of each chapter is identical between tracks (code solutions, feature descriptions, exercise instructions). The difference is in the explanatory prose, comparison boxes, and Rust Gym difficulty. Capstone chapters are shared.

---

## Chapter Structure Template

```markdown
# Chapter N: [Feature Name]

## What You'll Build
[Screenshot/demo of the feature]

## 🔦 Spotlight: [Rust Concept]
[Deep explanation of the concept]

> **Coming from JS/Python/Go?**
> [Side-by-side comparison]

## Building the Feature

### Exercise 1: [Step]
**Goal:** [What to build]
**Instructions:** [Steps]
<details><summary>Hint 1</summary>...</details>
<details><summary>Hint 2</summary>...</details>
<details><summary>Solution</summary>```rust ... ```</details>

### Exercise 2-4: [Same pattern]

## 🏋️ Rust Gym: [Spotlight Concept] Drills
[2-3 isolated exercises drilling ONLY the spotlight concept]

## 📊 DSA in Context: [Pattern Name]
> The code you wrote in Exercise 3 uses the [pattern] pattern.
> **Interview version:** [Problem description]
> **Bonus challenge:** [Harder variant]

## 🏗️ System Design Corner: [Topic]
> **Interview question:** "Design a [system]"
> **What we just built:** [Architecture mapping]
> **Talking points:** [Bullet points]

## What You Built
[Summary + what's visible in the app now]
```

---

## Curriculum: 26 Chapters (4 beginner-only + 18 feature + 1 design reflection + 3 capstone)

---

### Part 0: Programming Fundamentals (Beginner Track Only)

**These 4 chapters are ONLY in the beginner track. Experienced programmers skip directly to Chapter 1.**

#### Chapter 0.1: Your Workshop — Terminal, Files & Editors
- What is a computer doing when it runs a program?
- The terminal: your direct line to the machine (commands, navigation, creating files)
- Choosing an editor (VS Code setup with rust-analyzer)
- File system: directories, paths, extensions
- **Exercises:** Navigate terminal, create directories, open/edit files

#### Chapter 0.2: Your First Program — What Is Code?
- What is a programming language? (human instructions → machine instructions)
- Installing Rust and Cargo (step-by-step with screenshots)
- `cargo new hello` → `cargo run` → see "Hello, world!"
- Anatomy of a Rust program: `fn main()`, `println!`, semicolons, curly braces
- What is compilation? (Rust compiles to machine code vs. interpreted languages)
- **Exercises:** Modify hello world, print multiple lines, intentionally cause a compiler error and read the message

#### Chapter 0.3: Thinking Like a Programmer
- Breaking problems into steps (make a sandwich analogy → make a workout plan)
- Input → Processing → Output pattern
- Reading error messages: the compiler is your friend
- How to debug: read the error, isolate the problem, test a fix
- **Exercises:** Write pseudocode for a workout tracker, translate to print statements, debug intentional errors

#### Chapter 0.4: The Building Blocks — Variables, Functions & Loops in Rust
- **Variables:** storing data (`let`, `let mut`, types)
- **Types:** numbers (`i32`, `f64`), text (`String`, `&str`), booleans
- **Functions:** packaging logic (`fn`, parameters, return values)
- **Control flow:** `if/else`, `for` loops, `while`
- **Exercises:** Declare workout variables, write calculate_volume, loop through 5 sets, format and print results

**After Part 0:** Reader can write and run basic Rust programs. They understand variables, functions, loops, and the compiler. Ready for Chapter 1.

---

### Chapter 1: Hello, GrindIt!
**Feature:** Render a dark-themed "GrindIt" page with header and bottom nav
**🔦 Spotlight:** Variables, Types & the Leptos Toolchain
- Install Rust, cargo-leptos, set up the project
- Create the app shell with `view!` macro
- Add SCSS dark theme (CSS custom properties)
- Build bottom nav with mask-image SVG icons

**Rust Gym:** Variable declaration, String vs &str, println! formatting
**DSA:** —
**System Design:** SSR vs CSR vs Hydration — rendering strategy tradeoffs
**Reference:** `app.rs`, `_header.scss`, `_reset.scss`, `_themes.scss`

---

### Chapter 2: The Exercise Library
**Feature:** Display a categorized list of exercises (hardcoded data)
**🔦 Spotlight:** Structs & `impl` Blocks
- Define `Exercise` struct with name, category, scoring_type
- Implement `new()`, `summary()`, `is_weightlifting()` methods
- Render exercise cards grouped by category
- Color-coded category headers with count badges

**Rust Gym:** Struct memory layout, method vs associated function, &self borrowing
**DSA:** Data modeling — struct as record, memory layout implications
**System Design:** Domain modeling — how struct design maps to database schema
**Reference:** `db.rs` (Exercise struct), `exercises/exercise_card.rs`

---

### Chapter 3: Search & Filter
**Feature:** Real-time search bar, category filtering, expand/collapse cards
**🔦 Spotlight:** Closures & Iterators
- Search input bound to `RwSignal<String>` 💡(Signals spotlighted in context here — deep enough for practical use)
- Filter exercises with `.iter().filter().collect()` using closures
- Expand/collapse cards with `RwSignal<Option<String>>`
- Category grouping with `.filter().count()` for badge counts

**Rust Gym:** Closure captures (move, borrow), .map().filter().collect() chains, higher-order functions
**DSA:** **Linear search & string matching** — .contains() is O(n×m), when to optimize
**System Design:** Reactive systems — push vs pull update models
**Reference:** `exercises/mod.rs` (search logic, collapsed signal)

---

### Chapter 4: Exercise CRUD
**Feature:** Create, edit, and soft-delete exercises with forms and validation
**🔦 Spotlight:** Error Handling (Result, Option, ?)
- Create form with validation (Result for errors)
- Edit mode with `Option<Exercise>` (None = create, Some = edit)
- Soft delete with ownership check (Result for authorization errors)
- Display errors with toast notifications

**Supporting 💡:** Server functions (introduced here, spotlighted fully when DB connects in Ch 5)

**Rust Gym:** parse_weight → Option, chain ? operator, map_err patterns, clean_error()
**DSA:** **Null safety** — Option eliminates entire categories of bugs vs null in other languages
**System Design:** Soft delete vs hard delete — data retention tradeoffs
**Reference:** `exercises/server_fns.rs`, `auth/mod.rs` (clean_error)

---

### Chapter 5: Database Persistence
**Feature:** PostgreSQL with SQLx, migrations, connect exercises to real database
**🔦 Spotlight:** Async/Await & SQLx
- Set up PostgreSQL with Docker (`init_db.sh`)
- Write first migration (`create_exercises_table.sql`)
- `#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]` pattern
- Async query functions: `list_exercises_db()`, `create_exercise_db()`
- Global pool with `OnceLock<PgPool>`

**Rust Gym:** async fn, .await, tokio::join!, Result chaining in async contexts
**DSA:** **B-tree indexes** — how PostgreSQL indexes work, when to CREATE INDEX
**System Design:** **Connection pooling** — why pools exist, sizing, lazy vs eager connection
**Reference:** `db.rs`, `migrations/`, `scripts/init_db.sh`

---

### Chapter 6: Multi-Page Routing
**Feature:** All page routes, bottom nav highlighting, scroll reset
**🔦 Spotlight:** Modules & Project Structure
- Organize code into `src/pages/`, `src/components/`, `src/auth/`
- `mod.rs`, `pub use`, `pub(crate)` visibility
- `#[cfg(feature = "ssr")]` vs `#[cfg(feature = "hydrate")]` conditional compilation
- Set up `<Router>`, `<Routes>`, `StaticSegment` for all pages
- Active tab highlighting with `use_location().pathname`

**Rust Gym:** Create a module tree, re-export types, understand pub/pub(crate)/pub(super)
**DSA:** **Tree matching** — route resolution as prefix tree traversal
**System Design:** **URL design** — RESTful paths, query parameters, deep linking
**Reference:** `lib.rs`, `pages/mod.rs`, `app.rs` (routing)

---

### Chapter 7: User Authentication
**Feature:** Login page with Google OAuth, email/password, phone OTP
**🔦 Spotlight:** Enums & Pattern Matching
- `enum UserRole { Athlete, Coach, Admin }` with `rank()` method
- `match` for exhaustive handling, `if let` for Option
- `Display` trait implementation for UserRole
- OAuth flow, Argon2 password hashing, OTP verification
- Session management with tower-sessions

**Supporting 💡:** Traits (Display impl), Hashing (Argon2)

**Rust Gym:** Enum with methods, match with guards, nested pattern matching, From/Into conversions
**DSA:** **Hashing** — hash tables (HashMap) → password hashing (Argon2) → collision resistance
**System Design:** **Auth system design** — sessions vs tokens, OAuth flow, RBAC hierarchy, "design an auth system" interview answer
**Reference:** `auth/mod.rs`, `auth/oauth.rs`, `auth/password.rs`, `auth/session.rs`, `login/mod.rs`

---

### Chapter 8: WOD Programming
**Feature:** WOD creation with sections and movements, weekly calendar view
**🔦 Spotlight:** Complex Data Structures & Relationships
- Nested structs: `Wod` → `WodSection` → `WodMovement`
- Multi-table migrations with foreign keys
- Weekly calendar with date arithmetic (Julian Day Numbers)
- WOD form with dynamic section/movement management

**Rust Gym:** Nested struct access, Vec of structs, iteration over tree-like structures
**DSA:** **N-ary tree traversal** — WOD as a tree (root → sections → movements), DFS vs BFS for rendering nested UI
**System Design:** **Data modeling at scale** — normalization vs denormalization, when to join vs embed
**Reference:** `wod/mod.rs`, `wod/week_calendar.rs`, `wod/wod_form.rs`, migrations 06-08

---

### Chapter 9: Workout Logging & Scoring
**Feature:** Log workouts against WODs, score sections (time/rounds/weight), movement tracking
**🔦 Spotlight:** Traits & Generics
- `Serialize`/`Deserialize` for all models (serde)
- `impl IntoView` as return type for components
- `impl Fn() + Copy + 'static` for callback props
- Generic `Resource::new()` pattern for async data fetching

**Supporting 💡:** ServerAction dispatch, form state management

**Rust Gym:** Define a trait, implement for multiple types, derive macros, impl Trait in function signatures
**DSA:** **Strategy pattern** — different scoring types (ForTime, AMRAP, Strength) as strategies
**System Design:** **Form state management** — optimistic updates, pending states, error recovery
**Reference:** `log_workout/mod.rs`, `log_workout/wod_score_form.rs`, `log_workout/server_fns.rs`

---

### Chapter 10: History & Leaderboard
**Feature:** Workout history timeline, leaderboard rankings, streak calculation
**🔦 Spotlight:** Collections & Sorting Deep Dive
- `HashMap<String, Vec<WorkoutLog>>` for grouping by date
- Custom sorting for leaderboard (Rx first, then score, then time)
- Streak calculation using greedy consecutive-day matching
- `BTreeMap` for ordered date ranges
- Iterator chains for stats aggregation

**Rust Gym:** Sort with custom comparator, group_by pattern, consecutive sequence detection
**DSA:** **Greedy** — streak as "longest consecutive sequence" (LeetCode 128). **Custom comparators** — sorting with multiple criteria
**System Design:** **Leaderboard design** — real-time ranking at scale, Redis sorted sets, cache invalidation
**Reference:** `db.rs` (streak_days_db, leaderboard_db), `history/mod.rs`, `home/leaderboard.rs`

---

### Chapter 11: Reusable Components
**Feature:** Build DeleteModal, SingleSelect, MultiSelect, ExerciseCard as a component library
**🔦 Spotlight:** Ownership & Borrowing Deep Dive
- `RwSignal<T>` as shared mutable state (uses Arc internally)
- `move ||` closures capturing signals — why `Clone` is needed
- `impl Fn() + Copy + 'static` for callback props — understanding trait bounds
- `<Portal>` for escaping overflow (DOM reference management)
- The `clone → move` pattern that appears everywhere in Leptos

**Rust Gym:** Clone vs Copy, borrow checker exercises, lifetime in closures, signal sharing patterns
**DSA:** **Stack** — modal/overlay z-index as LIFO (last opened = first closed)
**System Design:** **Component library design** — API surface, prop contracts, composability
**Reference:** `components/delete_modal.rs`, `components/single_select.rs`, `components/multi_select.rs`

---

### Chapter 12: Profile & Admin
**Feature:** Profile page (edit name/email/gender, change password), admin panel (manage user roles)
**🔦 Spotlight:** Authorization & Role-Based Access Control
- `require_auth()` and `require_role(min_role)` guard functions
- Role hierarchy: Athlete < Coach < Admin with `rank()` comparison
- Ownership checks: "only creator or admin can delete"
- Conditional UI rendering based on role
- `default_role_for_new_user()` — first user is Admin

**Rust Gym:** Guard patterns with early return, role hierarchy as enum ordering, conditional logic
**DSA:** **Bit masking** (conceptual) — permission systems as bitmasks
**System Design:** **RBAC design** — role hierarchies, permission models, principle of least privilege
**Reference:** `auth/session.rs` (require_auth, require_role), `profile/mod.rs`, `admin/mod.rs`

---

### Chapter 13: Video Uploads
**Feature:** Video upload with validation, local + Cloudflare R2 storage backends
**🔦 Spotlight:** Smart Pointers (Arc) & Enum-Based Abstraction
- `enum StorageBackend { Local(PathBuf), R2 { bucket, ... } }` as strategy pattern
- `Arc<StorageBackend>` for sharing across async handlers
- Magic byte validation (ftyp, EBML, RIFF) for file security
- Multipart upload handling in Axum

**Rust Gym:** Arc::new and Arc::clone, enum dispatch with match, async methods on enums
**DSA:** **Strategy pattern** — enum dispatch as alternative to trait objects, O(1) type resolution
**System Design:** **File upload pipeline** — validation → storage → CDN, content-type enforcement, size limits
**Reference:** `storage.rs`, `routes/upload.rs`, `components/video_upload.rs`

---

### Chapter 14: PWA & WASM Interop
**Feature:** Service worker, theme toggle, PWA install banner, offline support
**🔦 Spotlight:** wasm_bindgen, web_sys, js_sys
- `#[wasm_bindgen(inline_js = "...")]` for calling JS from Rust
- `web_sys::Window`, `web_sys::Element` for DOM access
- `wasm_bindgen_futures::spawn_local()` for client-side async
- Service worker registration, cache strategies
- Theme toggle persisted in localStorage

**Rust Gym:** Call JS from Rust, access DOM APIs, spawn async tasks in WASM
**DSA:** **Cache invalidation** — service worker versioning, stale-while-revalidate strategy
**System Design:** **Caching strategies** — network-first vs cache-first, offline-first architecture
**Reference:** `pwa.rs`, `voice.rs`, `public/sw.js`, `public/manifest.json`

---

### Chapter 15: Configuration & Telemetry
**Feature:** Multi-environment config (YAML), structured logging, request tracing
**🔦 Spotlight:** Serde Deep Dive & Configuration Patterns
- `Settings` struct hierarchy with serde deserialization
- YAML base → env-specific overlay → env var override
- `TryFrom<String> for Environment` conversion
- Bunyan JSON formatter for structured logs
- TraceLayer middleware with request ID propagation

**Rust Gym:** Custom serde deserialization, TryFrom implementations, builder pattern
**DSA:** —
**System Design:** **Observability** — structured logging, distributed tracing, correlation IDs, "design a logging system"
**Reference:** `configuration.rs`, `telemetry.rs`, `configuration/base.yaml`

---

### Chapter 16: REST API Layer
**Feature:** Full REST API endpoints for exercises, WODs, workouts + Swagger UI
**🔦 Spotlight:** Axum Route Organization & API Design
- "Two doors, one database" — server functions for Leptos, REST for 3rd parties
- `Router::nest("/api/v1", api_routes)` for versioning
- `#[utoipa::path]` annotations for OpenAPI docs
- Middleware layer ordering (sessions → tracing → request IDs)
- `SwaggerUi` integration

**Both call the same `db.rs` functions — zero business logic duplication**

**Rust Gym:** Build an Axum handler, add OpenAPI annotations, test with curl
**DSA:** **Middleware as function composition** — Tower layers as composable functions (onion model)
**System Design:** **API gateway design** — versioning, rate limiting, documentation, multi-client architecture
**Reference:** `routes/mod.rs`, `routes/health_check.rs`, `routes/upload.rs`, `main.rs` (router setup)

---

### Chapter 17: Docker & Deployment
**Feature:** Production-ready multi-stage Docker image
**🔦 Spotlight:** Multi-Stage Builds & Build Optimization
- 4-stage Dockerfile: Chef → Planner → Builder → Runtime
- cargo-chef for dependency caching
- `SQLX_OFFLINE=true` for build without database
- Dart Sass installation for SCSS
- Production configuration with env var overrides

**Rust Gym:** Layer optimization exercises, image size comparison
**DSA:** —
**System Design:** **Containerization** — image layers, caching, multi-stage rationale, minimal runtime images
**Reference:** `Dockerfile`, `configuration/production.yaml`, `.sqlx/`

---

### Chapter 18: CI/CD
**Feature:** GitHub Actions pipelines for testing, linting, auditing, deploying
**🔦 Spotlight:** Quality Automation & Rust Tooling
- `cargo fmt --check` for formatting
- `cargo clippy -- -D warnings` for linting
- `cargo deny check` for license/vulnerability auditing
- PostgreSQL service container for integration tests
- Pre-commit hooks

**Rust Gym:** Fix clippy warnings, configure deny.toml, write a CI workflow
**DSA:** —
**System Design:** **CI/CD pipeline design** — test pyramid, deployment strategies, rollback plans
**Reference:** `.github/workflows/general.yml`, `.github/workflows/audit.yml`, `deny.toml`, `scripts/pre-commit`

---

### Chapter 18.5: Software Design Reflection
**Feature:** No new code — review and refactor earlier chapters through a design philosophy lens
**🔦 Spotlight:** *A Philosophy of Software Design* (Ousterhout) principles applied to GrindIt

Revisit code from previous chapters and evaluate through these lenses:

| Principle | GrindIt Example | Chapter |
|-----------|----------------|---------|
| **Deep modules** | `StorageBackend` — simple `.upload()` interface hides Local vs R2 complexity | Ch 13 |
| **Information hiding** | `db.rs` hides SQL behind clean async functions; components don't know about PostgreSQL | Ch 5 |
| **Define errors out of existence** | `clean_error()` strips internal prefixes; `Option<T>` eliminates null | Ch 4, 7 |
| **Strategic vs tactical** | Module structure (Ch 6) was strategic investment; pays off in every chapter after | Ch 6 |
| **Obvious code** | `UserRole::rank()` with match — intent is immediately clear | Ch 7 |
| **Comments for WHY** | Server function annotations explain business rules, not syntax | Throughout |
| **Complexity red flags** | Conjoined methods (functions doing two things), shallow abstractions | Refactor exercise |
| **Pull complexity downward** | `init_pool()` handles all connection complexity; callers just call `db()` | Ch 5 |

**Exercises:**
1. Identify a "shallow module" in your code and refactor it into a "deep module"
2. Find a function that handles errors it shouldn't — redesign the API to define the error out of existence
3. Review your comments: remove "what" comments, add "why" comments where missing
4. Refactor a "pass-through" function that adds no value

**This chapter is shared between both tracks.**

---

### Chapter 19: Coding Challenges (Capstone)
**All problems use GrindIt's fitness domain — not abstract arrays.**

| Problem | DSA Pattern | Description |
|---------|-------------|-------------|
| Workout Knapsack | **Dynamic Programming** | Maximize exercise benefit within 60-min time cap |
| Exercise Autocomplete | **Trie** | Prefix tree for exercise name search |
| Real-time Leaderboard | **Heap / Priority Queue** | Top-K scores as they stream in |
| Movement Prerequisites | **Topological Sort** | Order exercises by skill prerequisites |
| Next PR Finder | **Monotonic Stack** | Find next session where you lifted heavier |
| WOD Generator | **Backtracking** | Generate valid WOD combinations with constraints |
| Exercise Cache | **LRU Cache** | Frequently accessed exercise caching |
| Progression Path | **BFS/Dijkstra** | Shortest skill path from beginner to advanced |

**Format:** 4 exercises, each with brute force → optimized → complexity analysis

---

### Chapter 20: System Design Deep Dive (Capstone)

| Topic | Interview Question | GrindIt's Answer |
|-------|-------------------|-----------------|
| Design a Fitness Tracker | "Design SugarWOD/Wodify" | The entire GrindIt architecture |
| Auth at Scale | "Design multi-method auth for 1M users" | Ch 7 architecture + rate limiting, token rotation |
| Real-time Leaderboard | "Design a live leaderboard for 10K concurrent users" | DB-backed → Redis sorted sets → WebSocket push |
| File Upload Pipeline | "Design a video upload system" | StorageBackend → CDN, transcoding, thumbnails |
| Multi-tenant Gym Platform | "Extend for 1000 gyms" | Schema evolution, tenant isolation, RLS |
| Offline-first PWA | "Design offline sync" | Service worker → IndexedDB, conflict resolution |

**Format:** 15-min reads with architecture diagrams, capacity estimation, tradeoff discussion

---

### Chapter 21: Mock Interviews (Capstone)

1. **Coding Mock (45 min):** 2 problems using workout data. Clarify → brute force → optimize → code → test
2. **System Design Mock (45 min):** "Design GrindIt for 1M users." Requirements → capacity → HLD → deep dives → tradeoffs

---

## Spotlight Concept Progression

| Ch | Spotlight | First Exposure | Deep Dive | Reinforced In |
|----|-----------|---------------|-----------|---------------|
| 1 | Variables & Types | Ch 1 | Ch 1 | Every chapter |
| 2 | Structs | Ch 2 | Ch 2 | Ch 8, 9, 11 |
| 3 | Closures & Iterators | Ch 3 | Ch 3 | Ch 10, 11 |
| 4 | Error Handling | Ch 4 | Ch 4 | Ch 5, 7, 9 |
| 5 | Async/Await | Ch 4 (light) | Ch 5 | Ch 8, 9, 10 |
| 6 | Modules | Ch 2 (light) | Ch 6 | Every chapter |
| 7 | Enums & Matching | Ch 2 (light) | Ch 7 | Ch 12, 13 |
| 8 | Complex Structs | Ch 2 | Ch 8 | Ch 9 |
| 9 | Traits & Generics | Ch 7 (Display) | Ch 9 | Ch 11, 16 |
| 10 | Collections (deep) | Ch 2 (Vec) | Ch 10 | Ch 19 |
| 11 | Ownership & Borrowing | Ch 2 (light) | Ch 11 | Every chapter |
| 12 | Authorization patterns | Ch 7 (light) | Ch 12 | Ch 16 |
| 13 | Arc & Enum abstraction | Ch 5 (pool) | Ch 13 | Ch 16 |
| 14 | WASM interop | — | Ch 14 | — |
| 15 | Serde & Config | Ch 2 (derive) | Ch 15 | — |
| 16 | API design | Ch 5 (server fns) | Ch 16 | — |

Every concept is seen 3+ times: first exposure → spotlight → reinforcement.

---

## 📐 Design Insight Boxes (Ousterhout) — Woven into Chapters

| Chapter | Design Principle | Insight |
|---------|-----------------|---------|
| Ch 2 | **Obvious code** | Struct field names ARE documentation — no comments needed |
| Ch 4 | **Define errors out of existence** | Validation at creation prevents invalid state from existing |
| Ch 5 | **Pull complexity downward** | `db()` absorbs pool complexity — callers never think about OnceLock |
| Ch 6 | **Strategic programming** | Module structure investment pays dividends in every future chapter |
| Ch 8 | **Deep modules** | WOD hides tree complexity (sections → movements) behind flat interface |
| Ch 11 | **Information hiding** | DeleteModal doesn't know what it's deleting — just show/hide + confirm |
| Ch 13 | **Deep modules** | StorageBackend.upload() — one method, two implementations hidden inside |
| Ch 15 | **Complexity layers** | Config loading: base → env → overrides. Each layer hides those below |
| Ch 16 | **Pass-through elimination** | REST handlers and server functions both call db.rs — no pointless intermediary |

Format:
```markdown
> 📐 **Design Insight: Deep Modules** (Ousterhout, Ch. 4)
> `StorageBackend` has a simple interface — `.upload(name, bytes)` — but hides
> significant complexity inside. This is a *deep* module. A *shallow* module
> would expose filesystem/S3 details to every caller.
```

---

## DSA Pattern Coverage

### Organic (Chapters 1-18): ~18 patterns
HashMap, iterators, linear search, string matching, closures as HOF, Option/null safety, B-tree indexes, tree matching (routing), hashing (password + HashMap), N-ary tree traversal, strategy pattern, greedy (streak), custom comparators, sorting, stack (modals), cache invalidation, middleware composition, bit masking (conceptual)

### Capstone (Chapter 19): ~8 patterns
Dynamic programming, trie, heap/priority queue, topological sort, monotonic stack, backtracking, LRU cache, BFS/Dijkstra

**Total: ~26 DSA patterns + 6 system design deep dives + 2 mock interviews**

---

## Key Reference Files (from GrindIt codebase)

| File | Chapters |
|------|----------|
| `src/app.rs` | 1, 6, 14 |
| `src/db.rs` | 2, 5, 8, 9, 10, 16 |
| `src/lib.rs` | 6 |
| `src/main.rs` | 5, 7, 15, 16 |
| `src/auth/` | 7, 12 |
| `src/pages/exercises/` | 2, 3, 4 |
| `src/pages/wod/` | 8 |
| `src/pages/log_workout/` | 9 |
| `src/pages/history/` | 10 |
| `src/pages/login/` | 7 |
| `src/pages/profile/` | 12 |
| `src/pages/admin/` | 12 |
| `src/components/` | 11 |
| `src/configuration.rs` | 15 |
| `src/telemetry.rs` | 15 |
| `src/storage.rs` | 13 |
| `src/pwa.rs` | 14 |
| `src/voice.rs` | 14 |
| `src/routes/` | 16 |
| `Dockerfile` | 17 |
| `.github/workflows/` | 18 |
| `migrations/` | 5, 8 |
| `style/` | 1 |

---

## Versioning Strategy: Hybrid (Pinned Core + Living Evolution Log)

### Core Book (Pinned)
- Tag GrindIt repo at `book-v1` when book is first written
- All `code/chXX/` snapshots are based on this tag
- Core chapters (0-21) only change during **edition releases**
- Editions released when: Leptos major version, or accumulated evolution entries warrant a rewrite

### Evolution Log (Living)
```
src/evolution/
  README.md                          # Index of all evolution entries
  2026-04-auth-refactor.md           # Each entry: What Changed, Why, Code, Try It
  2026-05-meditation-feature.md
  2026-06-leptos-upgrade.md
```

Each entry teaches: what changed → why → before/after code → exercise for the reader.

### CI Sync Pipeline
```yaml
# .github/workflows/sync-check.yml (in gritwit-book repo)
# Runs weekly + on GrindIt main push (via repository_dispatch)
# 1. Compare key GrindIt files against book-v1 tag
# 2. cargo check all code/chXX/ snapshots
# 3. If drift detected → create GitHub issue with affected chapters
# 4. Auto-scaffold evolution entry with TODO markers
```

### Edition Release Process
1. Review accumulated evolution entries
2. Fold changes into core chapters
3. Update all `code/chXX/` snapshots
4. Re-tag GrindIt at `book-v2`
5. Archive old evolution entries as "v1 → v2 migration notes"

---

## Implementation Plan

### Step 1: Create `gritwit-book` repo
- mdBook structure with dual-track layout
- book.toml with conditional chapters (mdBook preprocessor or SUMMARY sections)
- All chapter file stubs for both tracks
- `code/` directory structure

### Step 2: Write Introduction
- What we're building (GrindIt overview)
- The triple goal: production Rust + DSA + system design
- "Choose your track" guide (beginner vs experienced)
- How to use this book (spotlight, Rust Gym, DSA boxes)

### Step 3: Write Part 0 (Beginner-only)
- Chapters 0.1-0.4: Programming fundamentals
- Gentle, no assumptions, lots of screenshots

### Step 4: Write Chapters 1-6 (Experienced track first)
- Setup through routing
- Ch 1 = rendered page (immediate visual payoff)

### Step 5: Adapt Chapters 1-6 for Beginner track
- Expand explanations, replace "Coming from JS?" with "Programming Concept" boxes
- Simplify Rust Gym drills
- Add more screenshots and expected-output blocks

### Step 6: Write Chapters 7-12 (both tracks in parallel)
- Auth through admin — heaviest feature content

### Step 7: Write Chapters 13-18 (both tracks in parallel)
- Uploads, PWA, config, API, Docker, CI/CD

### Step 8: Write Chapters 19-21 (Capstone — shared)
- Coding challenges, system design, mock interviews

### Step 9: Create code snapshots
- `code/ch00/` through `code/ch18/` compilable snapshots
- Verify each compiles

### Step 10: Review & polish
- Cross-reference with GrindIt repo
- mdbook build verification for both tracks
- "Going Deeper" links

---

## Verification Plan

1. Every `code/chXX/` compiles (`cargo build` for Part 0, `cargo leptos build` from Ch 1 onward)
2. Each exercise solution produces expected output
3. Rust Gym drills compile and run independently
4. DSA exercises include correct complexity analysis
5. `mdbook build` produces clean navigable site for BOTH tracks
6. Chapter 18's final code closely matches the GrindIt reference
7. Part 0 exercises work standalone (no Leptos dependency)
8. A true beginner can follow Part 0 → Ch 1 without external resources
