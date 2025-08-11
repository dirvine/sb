# Status Bar Test

This file is for testing the new persistent status bar at the bottom of the sb application.

## What's New

The main application now has a context-sensitive status bar that:

1. **Shows different commands based on current mode:**
   - **PREVIEW MODE**: Shows scrolling and editing commands
   - **EDITOR MODE**: Shows save/quit commands
   - **FILES**: Shows file navigation commands

2. **Has better visibility:**
   - Cyan background with black text (like file picker but cyan instead of yellow)
   - Bold text for better readability
   - Centered for aesthetic appeal

3. **Updates dynamically:**
   - Changes based on which pane has focus
   - Shows relevant commands for current context
   - Includes the app status message

## Testing

To test the new status bar:

1. Build: `cargo build`
2. Run: `./target/debug/sb status_bar_test.md`
3. Notice the cyan status bar at the bottom
4. Try switching focus with Tab - status bar updates
5. Press 'e' to enter editor mode - status bar changes
6. Press F2 to open file picker - picker has its own yellow status bar

The status bar provides a consistent, always-visible guide to available commands!