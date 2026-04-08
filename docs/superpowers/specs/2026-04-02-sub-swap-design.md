# sub-swap Design Specification

**Date:** 2026-04-02
**Status:** Approved
**Scope:** CLI/TUI tool for managing multiple `~/.codex/` profiles

---

## 1. Problem Statement

Codex CLI stores credentials in `~/.codex/auth.json` and provider/model configuration in `~/.codex/config.toml`. Users who operate multiple accounts (e.g., a corporate endpoint and a personal ChatGPT subscription) must manually rename files to switch between them. This is error-prone, leaves credentials in plaintext on disk, and provides no audit trail of which profile was used when.

`sub-swap` automates profile switching with encryption-at-rest for inactive credentials.

## 2. Goals and Non-Goals

### Goals

- Switch between named Codex profiles in a single command
- Encrypt all non-active profiles by default using AES-256-GCM
- Store encryption key in OS keychain (macOS, Linux, Windows)
- Provide an interactive TUI wizard for guided operation
- Run fully offline with zero network access
- Track profile metadata: name, notes, last-used timestamp

### Non-Goals

- Managing files beyond `auth.json` and `config.toml` (history, sessions, logs, caches are user-level)
- Remote sync or multi-machine profile sharing
- Full isolation between profiles (use containers/OS accounts for that)
- Password-based key derivation (key is machine-bound via OS keychain)

## 3. Storage Layout

```
~/.sub-swap/
├── config.json            # Global settings
├── profiles.json          # Plaintext metadata index
└── profiles/
    ├── work-prod/
    │   ├── auth.json.enc      # AES-256-GCM encrypted
    │   └── config.toml.enc
    ├── personal/
    │   ├── auth.json.enc
    │   └── config.toml.enc
    └── staging/
        ├── auth.json.enc
        └── config.toml.enc
```

### 3.1 config.json

```json
{
  "encryption_enabled": true
}
```

- `encryption_enabled` (boolean, default `true`): When `true`, all non-active profiles are stored encrypted. When `false`, profiles are stored as plaintext.
- Toggled via `sub-swap config set encryption true|false`.

### 3.2 profiles.json

```json
{
  "version": 1,
  "active_profile": "work-prod",
  "profiles": {
    "work-prod": {
      "name": "work-prod",
      "notes": "Production BrainCo endpoint",
      "created_at": "2026-04-02T16:00:00Z",
      "last_used": "2026-04-02T16:30:00Z"
    },
    "personal": {
      "name": "personal",
      "notes": "Official ChatGPT OAuth login",
      "created_at": "2026-04-02T16:00:00Z",
      "last_used": "2026-04-01T10:00:00Z"
    }
  }
}
```

- Never encrypted. Contains no secrets.
- `active_profile` tracks which profile is currently live in `~/.codex/`.
- `last_used` is updated on every `sub-swap use` invocation.

### 3.3 Profile Files

When `encryption_enabled` is `true`:
- Non-active profiles use `.enc` suffix: `auth.json.enc`, `config.toml.enc`
- Active profile files live in `~/.codex/` as plaintext (Codex must read them)
- The copy under `~/.sub-swap/profiles/<active>/` is also encrypted (synced on switch-away)

When `encryption_enabled` is `false`:
- All profile files stored as plaintext: `auth.json`, `config.toml`

## 4. Encryption

### 4.1 Algorithm

- **Cipher:** AES-256-GCM (authenticated encryption)
- **Nonce:** 96-bit, randomly generated per encryption operation via OS CSPRNG
- **File format:** `[12-byte nonce][ciphertext][16-byte GCM tag]`
- Each file (`auth.json`, `config.toml`) is encrypted independently

### 4.2 Key Management

- On first launch, a random 256-bit key is generated via the `rand` crate (backed by OS CSPRNG)
- Key stored in OS keychain:
  - **macOS:** Keychain Services (service: `sub-swap`, account: `encryption-key`)
  - **Linux:** secret-service D-Bus API (GNOME Keyring / KWallet)
  - **Windows:** Windows Credential Manager
- Accessed via the `keyring` crate which abstracts all three backends

### 4.3 Key Loss

If the keychain entry is missing (new machine, wiped keychain):
- `sub-swap` detects the missing key on any operation that requires decryption
- Warns that existing `.enc` files are **unrecoverable** without the original key
- Offers to generate a new key for future encryptions
- Existing encrypted profiles must be re-created from source

### 4.4 Encryption Lifecycle

1. **Switch (`sub-swap use <target>`):**
   - Decrypt target profile's `.enc` files in memory
   - Write plaintext to `~/.codex/auth.json` and `~/.codex/config.toml`
   - Read previous active profile from `~/.codex/`
   - Encrypt and write to `~/.sub-swap/profiles/<old>/` as `.enc` files
   - Update `profiles.json` (active_profile, last_used)

2. **View (`sub-swap decrypt <name>`):**
   - Decrypt profile files in memory
   - Print contents to stdout
   - No disk writes

3. **Encryption toggle (`sub-swap config set encryption true|false`):**
   - When toggling to `false`: decrypt all `.enc` files in `~/.sub-swap/profiles/`, write as plaintext, remove `.enc` files
   - When toggling to `true`: encrypt all plaintext profile files, write as `.enc`, remove plaintext files
   - Active profile in `~/.codex/` is never affected

## 5. Process Guard

Before switching profiles, `sub-swap` checks for running Codex processes:

- Uses the `sysinfo` crate to enumerate running processes (cross-platform, no shell-out)
- Searches for processes named `codex`
- **If found:** Blocks the switch with message:
  ```
  Codex is currently running (PID 12345). Switching profiles may cause unexpected behavior.
  Force switch? [y/N]
  ```
- **CLI override:** `sub-swap use <name> --force` bypasses the guard
- **TUI:** Shows warning with Force/Cancel options

## 6. CLI Interface

### 6.1 Commands

| Command | Description |
|---------|-------------|
| `sub-swap` | Launch interactive TUI wizard |
| `sub-swap list` | List profiles: name, notes, active marker |
| `sub-swap list -v` | Verbose: includes created_at and last_used timestamps |
| `sub-swap use <name>` | Switch to profile |
| `sub-swap use <name> --force` | Switch even if Codex is running |
| `sub-swap add <name>` | Import current `~/.codex/` auth+config as new profile |
| `sub-swap add <name> --from <path>` | Import auth.json and config.toml from a directory |
| `sub-swap remove <name>` | Delete a stored profile (interactive confirmation) |
| `sub-swap rename <old> <new>` | Rename a profile |
| `sub-swap note <name> <text>` | Set or update a profile's note |
| `sub-swap decrypt <name>` | Print decrypted profile contents to stdout (no disk write) |
| `sub-swap config set <key> <value>` | Update global setting (e.g., `encryption false`) |
| `sub-swap config show` | Display current global settings |

### 6.2 List Output

**Standard (`sub-swap list`):**
```
  work-prod    Production BrainCo endpoint
* personal     Official ChatGPT OAuth login
  staging      Staging API testing
```

`*` marks the active profile.

**Verbose (`sub-swap list -v`):**
```
  work-prod    Production BrainCo endpoint        Last used: 2026-04-02 16:30 UTC
* personal     Official ChatGPT OAuth login        Last used: 2026-04-02 17:00 UTC (active)
  staging      Staging API testing                 Last used: 2026-03-28 09:15 UTC
```

### 6.3 Error Cases

| Scenario | Behavior |
|----------|----------|
| `use` on already-active profile | No-op with message: "Profile 'X' is already active" |
| `use` on nonexistent profile | Error: "Profile 'X' not found. Run `sub-swap list` to see available profiles." |
| `add` with name that already exists | Error: "Profile 'X' already exists. Use a different name or `sub-swap remove X` first." |
| `remove` on active profile | Error: "Cannot remove the active profile. Switch to another profile first." |
| `rename` to existing name | Error: "Profile 'Y' already exists." |
| `decrypt` when encryption disabled | Prints files directly (they're already plaintext) |
| `decrypt` with missing keychain key | Error with guidance (see Section 4.3) |
| No `~/.codex/auth.json` on `add` | Error: "No auth.json found in ~/.codex/. Nothing to import." |

## 7. Interactive TUI Wizard

### 7.1 Main Menu

Launched with bare `sub-swap` command. Uses `ratatui` + `crossterm`.

```
┌─ sub-swap ──────────────────────────────────┐
│                                             │
│  Profiles:                                  │
│    work-prod    Production BrainCo endpoint │
│  > personal     Official ChatGPT OAuth      │
│    staging      Staging API testing         │
│                                             │
│  [Enter] Switch  [a] Add  [r] Rename        │
│  [d] Delete  [n] Note  [v] View  [q] Quit   │
│                                             │
└─────────────────────────────────────────────┘
```

- Arrow keys navigate the profile list
- Single-key actions from the bottom bar
- Active profile indicated with highlight/marker
- Process guard integrated into Switch action

### 7.2 Action Flows

**Switch:** Select profile -> process guard check -> confirm -> execute -> show success
**Add:** Prompt name -> prompt note (optional) -> import from `~/.codex/` -> confirm -> done
**Rename:** Select profile -> prompt new name -> confirm -> done
**Delete:** Select profile -> "Are you sure?" confirmation -> done
**Note:** Select profile -> prompt new note text -> done
**View:** Select profile -> show decrypted contents in scrollable view -> press any key to return

## 8. First Launch

Triggered when `~/.sub-swap/` or `profiles.json` does not exist. Runs before any other operation.

### 8.1 Flow

1. **Welcome:**
   ```
   Welcome to sub-swap - Codex profile manager.
   ```

2. **Detect existing config:** Check for `~/.codex/auth.json` and `~/.codex/config.toml`.
   - **If found:**
     ```
     Found existing Codex configuration in ~/.codex/.
     Save it as your first profile? [Y/n]
     ```
     - Prompt for profile name (suggest "default")
     - Prompt for optional note
   - **If not found:**
     ```
     No Codex configuration found in ~/.codex/.
     You can add profiles later with `sub-swap add <name>`.
     ```

3. **Encryption setup:**
   ```
   sub-swap encrypts inactive profiles by default using a key stored
   in your OS keychain. Generate encryption key now? [Y/n]
   ```
   - **Yes:** Generate key, store in keychain, set `encryption_enabled: true`
   - **No:** Set `encryption_enabled: false`, inform user they can enable later

4. **Create structure:** `~/.sub-swap/`, `~/.sub-swap/profiles/`, `config.json`, `profiles.json`

5. **Summary:**
   ```
   Setup complete.
     Profile saved: "default" (Production BrainCo endpoint)
     Encryption: enabled (key stored in macOS Keychain)
   
   Run `sub-swap` for the interactive wizard or `sub-swap --help` for commands.
   ```

## 9. Tech Stack

### 9.1 Dependencies (pinned)

| Crate | Version | Purpose | CVE Status |
|-------|---------|---------|------------|
| `clap` | `=4.6.0` | CLI argument parsing | Clean |
| `ratatui` | `=0.30.0` | TUI framework | Clean |
| `crossterm` | `=0.29.0` | Terminal backend | Clean |
| `aes-gcm` | `=0.10.3` | AES-256-GCM encryption | CVE-2023-42811 patched in this version |
| `rand` | `=0.10.0` | CSPRNG for key/nonce generation | Clean |
| `keyring` | `=3.6.3` | OS keychain access | Clean |
| `sysinfo` | `=0.38.4` | Process detection | Clean |
| `serde` | `=1.0.228` | Serialization | Clean |
| `serde_json` | `=1.0.149` | JSON handling | Clean |
| `toml` | `=1.1.2` | TOML read/write | Clean |
| `chrono` | `=0.4.44` | Timestamps | RUSTSEC-2020-0159 patched since 0.4.20 |
| `dirs` | `=6.0.0` | Home directory resolution | Clean |

### 9.2 No Network

- Zero network-capable crates in the dependency tree
- No async runtime (`tokio`, `async-std`)
- No HTTP clients (`reqwest`, `hyper`, `ureq`)
- No dynamic script loading
- Single-threaded, synchronous execution

### 9.3 Project Structure

```
src/
├── main.rs              # Entry point, clap CLI definition
├── cli.rs               # CLI command dispatch
├── tui/
│   ├── mod.rs           # TUI app state and event loop
│   ├── wizard.rs        # First-launch and interactive wizard
│   └── widgets.rs       # Profile list, action menu rendering
├── profile/
│   ├── mod.rs           # Profile struct, CRUD operations
│   ├── store.rs         # profiles.json and file I/O
│   └── switch.rs        # Switch logic: decrypt/copy/encrypt lifecycle
├── crypto/
│   ├── mod.rs           # Encrypt/decrypt functions
│   └── keychain.rs      # OS keychain read/write via keyring
├── guard.rs             # Codex process detection
└── config.rs            # Global config (~/.sub-swap/config.json)
```

## 10. Security Considerations

- **Encryption at rest:** All non-active credentials encrypted with AES-256-GCM by default
- **No plaintext temp files:** Decryption happens in memory only; `decrypt` command prints to stdout without touching disk
- **Key protection:** Encryption key never touches the filesystem; stored exclusively in OS keychain
- **File permissions:** All files under `~/.sub-swap/` created with mode `0600` (owner read/write only)
- **Process guard:** Prevents mid-session credential swaps that could cause auth failures or data leaks
- **No network surface:** Zero attack surface from network-related code
- **Memory:** Sensitive data (decrypted credentials, encryption key) should be zeroized after use where feasible (consider `zeroize` crate for future hardening)

## 11. Platform Support

| Platform | Keychain Backend | Process Detection | Status |
|----------|-----------------|-------------------|--------|
| macOS | Keychain Services | `sysinfo` | Primary |
| Linux | secret-service (GNOME Keyring / KWallet) | `sysinfo` | Supported |
| Windows | Credential Manager | `sysinfo` | Supported |

## 12. Future Considerations (Out of Scope)

- Profile export/import for machine migration (encrypted archive)
- Shell completions (`clap_complete`)
- `zeroize` crate integration for sensitive memory wiping
- Profile groups or tagging
- Auto-switch based on working directory
