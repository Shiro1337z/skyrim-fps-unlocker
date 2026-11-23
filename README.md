# Skyrim FPS Unlocker

A small, dependency-free command-line utility that unlocks the framerate in
**The Elder Scrolls V: Skyrim Special Edition** by applying the well-known,
community-documented ini tweaks safely:

- `Skyrim.ini` `[Display]` `bLockFramerate=0`
- `Skyrim.ini` `[Display]` `iVSyncPresentInterval=0`
- `Skyrim.ini` `[HAVOK]` `fMaxTime=1/<fps>` and `fMaxTimeComplex=2/<fps>`
- `SkyrimPrefs.ini` `[Display]` `iVSyncPresentInterval=0`

It backs up your original ini files (`Skyrim.ini.bak`, `SkyrimPrefs.ini.bak`)
before touching them and can revert them with a single flag. No game files are
modified, no DLL injection, no reverse engineering of the executable.

## Why this exists

Skyrim Special Edition caps its framerate at 60 FPS by default. Above 60 FPS the
Havok physics simulation runs too fast, which causes items to fly around, quests
to break, and general instability. The community fix is to disable the frame
limiter (`bLockFramerate=0`) and tell Havok the new frame time (`fMaxTime`).
This tool automates that and computes the correct `fMaxTime` for your target
framerate.

> Note: Skyrim's engine is only truly stable up to ~60 FPS for some scripts.
> Higher framerates can still break certain quests/mods. For the most robust
> high-FPS experience, consider [SSE Display Tweaks](https://github.com/SlavicPotato/SSEDisplayTweaks)
> (an SKSE plugin) which does dynamic physics scaling. This tool is the simpler,
> ini-only approach.

## Build

Requires Rust (stable). No external dependencies.

```sh
cargo build --release
```

The binary will be at `target/release/skyrim-fps-unlocker.exe` (Windows).

## Usage

```sh
# Apply with default 144 FPS target
skyrim-fps-unlocker.exe --yes

# Apply for a 240 Hz monitor
skyrim-fps-unlocker.exe --fps 240 --yes

# Show current state without changing anything
skyrim-fps-unlocker.exe --detect

# Restore original ini files
skyrim-fps-unlocker.exe --revert

# Apply and launch the game
skyrim-fps-unlocker.exe --fps 144 --yes --launch
```

Options:

| Flag | Description |
|------|-------------|
| `--fps <n>` | Target framerate (default 144). Used for Havok `fMaxTime`. |
| `--revert` | Restore ini files from backups. |
| `--detect` | Print current ini state and exit. |
| `--launch` | Launch `SkyrimSE.exe` after applying. |
| `--ini-dir <p>` | Directory containing `Skyrim.ini` / `SkyrimPrefs.ini`. |
| `--game-dir <p>` | Skyrim SE install directory (for `--launch`). |
| `--yes`, `-y` | Skip confirmation prompt. |
| `--help`, `-h` | Show help. |

By default the ini directory is
`%USERPROFILE%\Documents\My Games\Skyrim Special Edition\`.

## Safety

- Original files are copied to `*.bak` only if a backup does not already exist,
  so re-running the tool will not overwrite your original backup.
- `--revert` restores from backup and removes the backup.
- Only ini text files are touched. The game executable is never modified.

## License

MIT - see [LICENSE](LICENSE).