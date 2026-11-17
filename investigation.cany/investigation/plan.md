# Plan

1. [done] Map the workspace and confirm toolchain (cargo, egui versions) — Toolchain confirmed: cargo 1.100, eframe 0.32.3 pinned.
2. [done] Design the app structure: theme, state model, canvas, library, wiring, simulation, instruments — Module split in app/src: model, theme, symbols, routing, sim, canvas, panels, instruments, ui_kit.
3. [done] Implement schematic canvas with dot grid, zoom/pan, snapping, select/move/rotate/mirror — Verified visually in the screenshot run: grid, pan/zoom pill, toolbar, selection, rotate/mirror/delete shortcuts in the status bar.
4. [done] Implement component library sidebar with search, categories and symbol previews — Sidebar shows search, category chips and 18 parts with drawn symbol previews; verified in screenshot.
5. [done] Implement smart orthogonal wiring with editable routes and pin snapping — Orthogonal routed wires with draggable segments; demo circuit draws 15 wires cleanly.
6. [done] Implement simulation engine with play/pause/step/stop and animated red/blue flow — Transport bar (stop/pause/step), time and speed control; wires colored red HIGH / blue LOW / amber analog in the screenshot.
7. [done] Implement floating virtual instruments (voltmeter, ammeter, logic probe, oscilloscope) — Voltmeter and oscilloscope open in floating windows by default; ammeter and logic probe toggle from the top bar.
8. [done] Add --screenshot mode, build, verify PNG, run rebuild_check — --screenshot saves a 1920x1080 PNG (unfocused, always-on-bottom on Windows); rebuild_check: build passed, app-screenshot passed.
