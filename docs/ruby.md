## 💎 Ruby Runtime Installer for Rustyhook CLI

This document outlines what your agent should build: a **Ruby prebuilt binary installer** that can be integrated into the `rustyhook` CLI tool as part of its environment setup. It should be implemented in Rust, but **this is not code**, only functional specifications.

---

### 🧩 Purpose
Create a utility that:
- Downloads a precompiled Ruby binary (from official sources or trusted third-party distributions)
- Extracts it to a local subdirectory (e.g. `.runtime/ruby/{version}`)
- Verifies the installation by running `ruby --version`
- Is invoked by `rustyhook`'s setup/init code as a prerequisite step for environments that depend on Ruby

---

### 🎯 Functional Requirements

This system must support all major platforms, including Windows. On Windows, `.zip` archives should be downloaded instead of `.tar.gz` or `.tar.xz`, and executable paths should reflect `.exe` suffixes and native directory conventions. Paths should be normalized to ensure compatibility across OS boundaries.

#### 1. **Target Ruby Versions**
- Accept a specific Ruby version (e.g. `3.2.2`) passed from config or CLI flags
- If not provided, determine the version from a standard dotfile such as `.ruby-version` in the project root

#### 2. **Platform Detection**
- Dynamically determine platform triple (`linux-x64`, `darwin-arm64`, `win-x64`, etc) using system information

#### 3. **Download Prebuilt Binary**
- Pull `.tar.gz`, `.tar.xz`, or `.zip` from a preconfigured URL template (e.g. [rubyinstaller.org](https://rubyinstaller.org/downloads/) for Windows, ruby-lang.org mirrors, or other trusted sources)
- Save to a cache or local working directory

#### 4. **Extraction**
- Extract into `.runtime/ruby/{version}/`
- Final path should include:
    - Ruby binary at `.runtime/ruby/{version}/bin/ruby`
    - Do not assume or require the presence of `gem`, `bundle`, or other Ruby tools
    - Systems depending on Ruby should function without relying on global Ruby environments or tooling

#### 5. **Verification**
- Run `ruby --version` from the installed path
- Log or return the installed version string

---

### 🔗 Integration Expectations

- This logic will not be standalone. It is invoked by `rustyhook` as part of its environment bootstrap.
- Prefer returning `Result<PathBuf, Error>` from the installer interface so the caller can handle success/failure
- No hardcoded version strings

---

### 💡 Optional Enhancements
- Cache previously downloaded archives.
- Support a `--force` option to reinstall even if a version is already present.
- Allow fallbacks or mirrors if primary binary host is unavailable.
- Support integrity check via SHA256 or similar.

---

Let me know if the Ruby version manager should also support local shims or dependency isolation (e.g. through `chruby`-like wrappers or vendored gem directories).
