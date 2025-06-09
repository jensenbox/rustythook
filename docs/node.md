## 📦 Node.js Runtime Installer for Rustyhook CLI

This document outlines what your agent should build: a **Node.js prebuilt binary installer** that can be integrated into the `rustyhook` CLI tool as part of its environment setup. It should be implemented in Rust, but **this is not code**, only functional specifications.

---

### 🧩 Purpose
Create a utility that:
- Downloads a precompiled Node.js binary (from nodejs.org)
- Extracts it to a local subdirectory (e.g. `.runtime/node/{version}`)
- Verifies the installation by running `node --version`
- Is invoked by `rustyhook`'s setup/init code as a prerequisite step for environments that depend on Node

---

### 🎯 Functional Requirements

This system must support all major platforms, including Windows. On Windows, `.zip` archives should be downloaded instead of `.tar.xz`, and executable paths should reflect `.exe` suffixes and native directory conventions. Paths should be normalized to ensure compatibility across OS boundaries.

#### 1. **Target Node.js Versions**
- Accept a specific Node.js version (e.g. `20.11.1`) passed from config or CLI flags
- If not provided, determine the version from a standard dotfile such as `.node-version` or `.nvmrc` in the project root (e.g. `20.11.1`) passed from config or CLI flags

#### 2. **Platform Detection**
- Dynamically determine platform triple (`linux-x64`, `darwin-arm64`, `win-x64`, etc) using system information

#### 3. **Download Prebuilt Binary**
- Pull `.tar.xz` or `.zip` from `https://nodejs.org/dist/v{version}/node-v{version}-{platform}.tar.xz`
- Save to a cache or local working directory

#### 4. **Extraction**
- Extract into `.runtime/node/{version}/`
- Final path should include:
    - Node binary at `.runtime/node/{version}/node-v{version}-{platform}/bin/node`
    - Do not assume or require the presence of `npm` or `npx` binaries
    - Systems depending on Node should function without relying on existing global tooling like `npm` or `npx``.runtime/node/{version}/`
- Final path should include:
    - Node binary at `.runtime/node/{version}/node-v{version}-{platform}/bin/node`
    - Optionally, `npm`/`npx` if available

#### 5. **Verification**
- Run `node --version` from the installed path
- Log or return the installed version string

---

### 🔗 Integration Expectations

- This logic will not be standalone. It is invoked by `rustyhook` as part of its environment bootstrap.
- Prefer returning `Result<PathBuf, Error>` from the installer interface so the caller can handle success/failure
- No hardcoded version strings

---

### 💡 Optional Enhancements
- Cache previously downloaded archives.
- Support a `--force` option to reinstall even if a version is present


