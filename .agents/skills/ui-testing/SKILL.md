---
name: ui-testing
description: Comprehensive UI/UX testing, touch gesture simulation, Android screen layout verification, and user interface responsiveness auditing.
---

# UI/UX Testing & Auditing Skill

Use this skill when auditing mobile user interfaces, testing multitouch gestures, verifying virtual on-screen controls, and benchmarking UI frame rates and layout boundaries.

## Key Testing Areas

1. **Touch & Multitouch Simulation**:
   - Simultaneous multi-finger gestures (pinch-to-zoom, dual joystick input, button chords).
   - Boundary hit testing (edges, corners, overlapping touch zones, deadzones).

2. **Resolution & Density Scalability**:
   - Test UI layouts on small budget phones (e.g., 720x1520), tablets (1920x1200), and foldable devices.
   - Verify that button hit-targets meet accessibility standards (minimum 48x48 dp).

3. **Performance & Frame Pacing**:
   - Audit UI render loops to prevent UI thread stalls.
   - Ensure touch overlays do not trigger memory allocations or garbage collection during frame updates.
