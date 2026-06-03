# F-Droid Reproducible Build Strategy

> **Goal**: Ensure every GuitarHub APK published on F-Droid can be independently
> verified to match the source code at a given git tag.

## Reference

- [F-Droid Reproducible Builds Guide](https://f-droid.org/docs/Reproducible_Builds/)
- [Gradle Reproducible Builds Plugin](https://github.com/izacus/gradle-reproducible-builds)
- [Tauri 2 Android Build Guide](https://v2.tauri.app/start/build/android/)

---

## Build Environment

Use a **pinned Docker image** to eliminate host OS drift:

```dockerfile
FROM ubuntu:22.04@sha256:<pinned-sha256>

RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    file \
    git \
    libssl-dev \
    pkg-config \
    python3.12 \
    python3.12-venv \
    && rm -rf /var/lib/apt/lists/*

# Rust — pinned toolchain version
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
    -y --default-toolchain 1.83.0
ENV PATH="/root/.cargo/bin:${PATH}"

# Android NDK — pinned from Tauri 2 required revision
RUN mkdir -p /opt/android-sdk && \
    curl -fsSL https://dl.google.com/android/repository/\
commandlinetools-linux-11076708_latest.zip -o cmdline-tools.zip && \
    unzip cmdline-tools.zip -d /opt/android-sdk/ && \
    rm cmdline-tools.zip
ENV ANDROID_HOME="/opt/android-sdk"
ENV ANDROID_NDK_HOME="/opt/android-sdk/ndk/<pinned-ndk-version>"

WORKDIR /build
```

**Key points**:
- Pin the **base OS image** by SHA256 digest, not just tag.
- Pin the **Rust toolchain version** (use `rust-toolchain.toml` in the repo).
- Pin the **Android SDK / NDK / Command-Line Tools** exact versions.
- Pin the **Tauri CLI version** (`cargo install tauri-cli --version <pinned>`).

---

## Dependency Pinning

### Rust (Cargo)

- `Cargo.lock` is **committed to git**. Every build uses exact dependency trees.
- `rust-toolchain.toml` pins the Rust compiler version:

  ```toml
  [toolchain]
  channel = "1.83.0"
  components = ["rust-src", "rust-std"]
  targets = ["aarch64-linux-android", "x86_64-linux-android"]
  ```

- Vendoring (optional, for maximum determinism):

  ```bash
  cargo vendor vendor/
  ```

  Then configure `.cargo/config.toml`:

  ```toml
  [source.crates-io]
  replace-with = "vendored-sources"

  [source.vendored-sources]
  directory = "vendor/"
  ```

### Python (scraper — not in APK, but needed for build scripts)

- `requirements.txt` pins exact versions (`package==1.2.3`).
- Use `pip freeze` to generate or update:

  ```bash
  pip freeze > requirements.txt
  ```

---

## Build Command Chain

### 1. Prepare the source

```bash
git checkout v<version>
git submodule update --init --recursive
```

### 2. Verify the tag matches the expected commit

```bash
git verify-tag v<version>          # if signed
git diff --exit-code HEAD          # no uncommitted changes
```

### 3. Build the Tauri Android APK

```bash
# Install pinned Tauri CLI
cargo install tauri-cli --version <pinned>

# Build Android APK in release mode
cargo tauri android build --apk
```

The APK will be at:
`src-tauri/gen/android/app/build/outputs/apk/universal/release/`

> Tauri 2 produces a **universal APK** by default. For per-architecture APKs,
> add `--target aarch64 --target x86_64` etc.

### 4. Align the APK

```bash
zipalign -p -f 4 app-universal-release-unsigned.apk GuitarHub-<version>.apk
```

---

## How to Verify Reproducibility

1. **Build twice** in the same Docker container, from the same git tag, with the
   same dependency tree:

   ```bash
   # First build
   docker build -t guitarhub-builder .
   docker run --rm -v $(pwd):/build guitarhub-builder \
       bash -c "cd /build && cargo tauri android build --apk && \
                cp src-tauri/gen/android/app/build/outputs/apk/universal/release/*.apk /tmp/build1.apk"

   # Clean rebuild in a fresh container
   docker run --rm -v $(pwd):/build guitarhub-builder \
       bash -c "cd /build && cargo clean && cargo tauri android build --apk && \
                cp src-tauri/gen/android/app/build/outputs/apk/universal/release/*.apk /tmp/build2.apk"

   # Compare hashes
   sha256sum /tmp/build1.apk /tmp/build2.apk
   ```

2. The two APKs MUST produce **identical SHA-256 hashes**. If they differ,
   identify the non-deterministic source (see Known Challenges below).

3. Include the **expected hash** in the F-Droid metadata (`build.gradle` or
   `build.yaml` in the fdroidserver metadata repo):

   ```yaml
   Builds:
     - versionName: <version>
       commit: v<version>
       output: app-universal-release-unsigned.apk
       sha256: <expected-hash>
   ```

---

## Known Challenges

### 1. Gradle non-reproducibility

Gradle builds embed timestamps, build IDs, and other non-deterministic metadata
in the APK by default.

**Mitigations**:

- Use the [`gradle-reproducible-builds`](https://github.com/izacus/gradle-reproducible-builds)
  Gradle plugin:

  ```kotlin
  // build.gradle.kts
  plugins {
      id("org.gradle.android.reproducible-builds") version "<version>"
  }
  ```

  This sets `isReproducibleOutput = true` on all APK tasks.

- Set `ANDROID_LINT_DISABLE=true` to avoid lint timestamp variance.

- In `gradle.properties`:

  ```properties
  android.builder.sdkDownload=false
  android.useAndroidX=true
  org.gradle.daemon=false
  org.gradle.parallel=false
  org.gradle.configureondemand=false
  ```

### 2. Rust compiler versions

Different Rust compiler versions can produce different machine code even from
the same source — different inlining, register allocation, etc.

**Mitigations**:

- Pin the **exact Rust toolchain** in `rust-toolchain.toml`.
- The Docker image installs that precise version.
- CI builds also use the same `rust-toolchain.toml` — so CI and F-Droid builds
  match.

### 3. Cargo dependency resolution

If `Cargo.lock` is absent or stale, `cargo build` may resolve different versions.

**Mitigation**: Commit `Cargo.lock` and verify it is up to date:

```bash
cargo check --locked   # fails if Cargo.lock is stale
```

### 4. Android SDK / NDK version drift

The Android build tools are downloaded at build time and can change.

**Mitigations**:

- Pre-install the SDK/NDK in the Docker image at pinned versions.
- Use `sdkmanager` to install only the exact required packages:

  ```bash
  sdkmanager "platforms;android-34" \
             "build-tools;34.0.0" \
             "ndk;27.0.12077973" \
             "cmake;3.22.1"
  ```

### 5. Timestamps in APK

Even with `isReproducibleOutput = true`, some embedded files may carry
timestamps.

**Mitigation**: Use `apktool` to decode and diff the two APKs:

```bash
apktool d build1.apk -o build1/
apktool d build2.apk -o build2/
diff -r build1/ build2/
```

If only `build-metadata.properties` differs, the APK is functionally
reproducible and F-Droid accepts it.

---

## Docker-based Builds (Recommended)

Create `Dockerfile.fdroid` in the repo root:

```dockerfile
FROM ubuntu:22.04@sha256:<pinned>

# Install system deps
RUN apt-get update && apt-get install -y \
    build-essential curl file git libssl-dev pkg-config python3.12 python3.12-venv \
    openjdk-17-jdk-headless \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo PATH=/opt/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain none

# Install Android SDK
ENV ANDROID_HOME=/opt/android-sdk
RUN mkdir -p $ANDROID_HOME && \
    curl -fsSL https://dl.google.com/android/repository/\
commandlinetools-linux-11076708_latest.zip -o /tmp/cmdline-tools.zip && \
    unzip /tmp/cmdline-tools.zip -d $ANDROID_HOME && \
    rm /tmp/cmdline-tools.zip && \
    yes | $ANDROID_HOME/cmdline-tools/bin/sdkmanager --sdk_root=$ANDROID_HOME \
        "platforms;android-34" \
        "build-tools;34.0.0" \
        "ndk;27.0.12077973" \
        "cmake;3.22.1" && \
    rm -rf $ANDROID_HOME/.temp

WORKDIR /build
COPY . .

# Install Rust toolchain (pinned)
RUN rustup toolchain install 1.83.0 && \
    rustup target add aarch64-linux-android x86_64-linux-android --toolchain 1.83.0

# Install pinned Tauri CLI
RUN cargo install tauri-cli --version 2.1.3

# Build
RUN cargo tauri android build --apk
```

This Dockerfile serves as the **single source of truth** for both CI and
F-Droid builds. Apply for F-Droid with the `Build:` entry pointing to this
Dockerfile.

---

## F-Droid Metadata Entry (fdroidserver)

When submitting to F-Droid, the `build.yaml` (or `build.gradle` equivalent) in
the [fdroidserver metadata repo](https://gitlab.com/fdroid/fdroiddata) should
reference:

```yaml
Builds:
  - versionName: 0.1.0
    commit: v0.1.0
    submodules: true
    output: app-universal-release-unsigned.apk
    build: |
      docker build -t guitarhub-builder -f Dockerfile.fdroid .
      docker run --rm guitarhub-builder
    scanclear: true
    ndk: r27
```

> **Note**: F-Droid requires the build to succeed **without network access**
> after the initial checkout. Vendoring Rust crates and Python packages is
> strongly recommended.
