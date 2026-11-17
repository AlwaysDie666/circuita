# Brief

**Question:** Rebuild a Proteus ISIS-style electronic circuit and schematic simulator as a native desktop app (Rust + eframe/egui) with a modern high-contrast matte-black dark UI and orange accents: interactive schematic canvas, component library sidebar with previews, smart orthogonal wiring, real-time simulation with animated current and logic flow, and floating virtual instruments.

**Purpose:** The person wants a modern replacement for Proteus Design Suite (ISIS) schematic capture and simulation, usable without coding knowledge.

**Constraints:**
- Clean room: no original target exists to open.
- Stack fixed by task instructions: Rust with eframe/egui, cargo project at app/Cargo.toml.
- Only app/ may be written.
- No dev servers or watchers; cargo build must succeed.
- Support --screenshot <file.png>: draw ~120 frames, save PNG, unfocused and behind other windows on Windows.

**Done when:** cargo build succeeds, screenshot PNG shows a complete schematic-editor UI, and rebuild_check reports no fixable differences.
