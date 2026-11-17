# Handoff

Circuita (app/, cargo bin "circuita", eframe/egui 0.32.3) is built and passing: cargo build succeeds; rebuild_check reports build passed and app-screenshot passed (1920x1080 PNG via --screenshot; window unfocused and AlwaysOnBottom on Windows).

This turn's fix: the oscilloscope floating window's default position covered the canvas zoom pill at the bottom-right (only a sliver showed). Raised its default in instruments.rs show() to rect.bottom()-14-396 and nudged x to rect.right()-14-390 so the window sits fully inside the canvas. Verified in a fresh screenshot: zoom pill fully visible, scope clear of it.

Known harmless quirks, not fixed: ~70 compiler warnings (mostly f32-literal fallback that cargo fix could apply; skipped because the workspace has no VCS to revert with); ground-net label numbering (GND4/GND5/GND6) can shift between runs if names come from hash-map order — cosmetic only.

If resumed: no open unknowns. Possible follow-ups to ask the person: save/load projects, editable component values, deeper MCU behaviour. Note the engine is a didactic DC/AC toy solver, not SPICE-accurate.
