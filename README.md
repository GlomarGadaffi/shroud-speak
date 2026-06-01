# shroud

> **Working name — not final.** Placeholder pending a decision (see *Open Decisions* below).
> Candidates so far: `shroud`, `nightwire`, `cant` (as in *cant* / can't), `deadkey`.

Encrypted push-to-talk voice over Tor onion services, as a single self-contained binary.

`shroud` is a ground-up Rust rewrite of **TerminalPhone** (a Bash orchestrator that
shelled out to `tor`, `socat`, `openssl`, `opusenc/opusdec`, `sox`, and ALSA tools).
The rewrite collapses that pipeline into one async process that owns every primitive
in memory and **never shells out and never writes audio to disk**.

## What it is

- A walkie-talkie. Hold a key, talk; release, listen. Two parties, or N via relay.
- Addressed and transported entirely over Tor v3 onion services. No IPs exchanged.
- End-to-end encrypted *above* Tor with a Noise handshake keyed by a shared secret,
  so the circuit crypto is not the only thing standing between you and a listener.

## Why rewrite it

| Bash / TerminalPhone | shroud |
| --- | --- |
| ~10 external binaries, FIFOs, fd juggling, `socat` | one static binary, in-process |
| audio chunks hit disk as `.tmp`/`.opus`/`.enc` | RAM-only ring buffers, zeroized keys |
| AES-256-CBC + ad-hoc HMAC + PBKDF2-per-chunk | Noise transport (AEAD, forward secrecy) |
| `AUDIO:<base64>\n` text framing (+33%, leaks length) | length-prefixed binary frames, paddable |
| external `tor` daemon + `torrc` + hostname file on disk | embedded [arti], optional ephemeral in-memory onion |
| `install_deps` + per-package-manager branches | nothing to install |

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for the full mapping and rationale.

## Status

**Pre-alpha / design.** Nothing here runs yet. The first milestone is a de-risking
spike (M0) that proves the core premise — hosting and self-dialing an onion service
in-process with no external `tor`. See [`docs/ROADMAP.md`](docs/ROADMAP.md).

## Platforms (target)

Linux (musl static), macOS, Windows, Android/Termux. One codebase, `cargo build`.
Audio capture/playback abstracted via `cpal` (ALSA / CoreAudio / WASAPI / Oboe).

## Threat model

This is a security tool; read [`docs/THREAT_MODEL.md`](docs/THREAT_MODEL.md) before
trusting it with anything. Short version: it targets network-adversary confidentiality
and metadata resistance, **not** endpoint compromise. A rooted phone or a keylogger
defeats it, as it defeats everything.

## Open Decisions

These are deliberately unresolved and tracked here until closed:

- [ ] **Name.** (see top)
- [ ] **UI / process model:** TUI monolith vs. headless daemon + thin clients.
      Leaning daemon — it lets hardware (pocket-dial, gatekeeper-class devices) be
      first-class clients of the same engine. See ARCHITECTURE §UI.
- [ ] **Tor layer:** [arti] (recommended) vs. linking C-tor. Gated on confirming
      arti's onion-service vanguards / DoS hardening is compiled in and on.
- [ ] **Repo visibility:** start private through M0–M2, flip public at a tagged
      milestone? Or public from day one (consistent with prior projects)?
- [ ] **License:** inheriting MIT from TerminalPhone unless we want copyleft.

## License

MIT (placeholder — see [`LICENSE`](LICENSE)). Inherited from TerminalPhone.

[arti]: https://gitlab.torproject.org/tpo/core/arti
