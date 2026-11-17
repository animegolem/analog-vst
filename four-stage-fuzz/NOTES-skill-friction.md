# Skill/template friction

- A global Cargo config directs builds to shared `~/.cargo-target`; simultaneous projects overwrite identically named `render` and `plugin` binaries and `install.sh` expects local `target/bundled`. Set `CARGO_TARGET_DIR="$PWD/target"` for the entire run, including harness subprocesses. Otherwise validation can fail with missing control placeholders or validate the wrong project; install can fail copying bundles.
- `SKILL.md` step 6 recommends a foreground 3600s terminal timeout. This runtime promotes foreground timeouts over 600s to background jobs; document `background=true`, notify, and process wait. Subagent jobs require a wait or explicit handoff before exit.
- CPU scales severely when two nonlinear feedback stages couple multiple transistor and diode-pair devices. Recommend benchmark immediately on first compile, and report concurrent-load conditions.
- Schematic article prose and component list can disagree with its own diagram. Cite the exact authoritative drawing and record each disagreement; do not silently mix them.
