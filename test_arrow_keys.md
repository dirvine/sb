# Arrow Key Pane Switching Test

## What's New in v0.3.3

The sb application now supports using **left and right arrow keys** to switch between panes when in Preview mode or File chooser mode!

### How it Works

#### In Preview Mode (right pane):
- **Left Arrow (←)**: Switch focus to the Files pane (if visible)
- **Right Arrow (→)**: Scroll content horizontally

#### In Files Mode (left pane):
- **Right Arrow (→)**: 
  - If on a directory: Expand the directory
  - If on a file: Switch focus to Preview/Editor pane
- **Left Arrow (←)**: Collapse the current directory

#### In Editor Mode:
- Arrow keys continue to move the cursor as normal (no pane switching)

### Testing Instructions

1. **Build the app**: `cargo build`
2. **Run the app**: `./target/debug/sb test_arrow_keys.md`
3. **Test pane switching**:
   - Press Tab to ensure you're in Preview mode
   - Press ← to switch to Files pane
   - Navigate to a file with ↑↓
   - Press → to switch back to Preview
   - Notice the status bar updates to show available commands

### Status Bar Updates

The status bar now shows when arrow keys can switch panes:
- **Preview Mode**: Shows "← files" when left pane is visible
- **Files Mode**: Shows "→ preview" to indicate right arrow switches panes

This makes navigation more intuitive and keyboard-centric!

## Benefits

- **Faster navigation**: No need to use Tab for simple left/right switching
- **More intuitive**: Arrows naturally suggest direction of focus movement
- **Preserves functionality**: In editor mode, arrows still move cursor as expected
- **Smart behavior**: Right arrow on files tries to expand first, then switches panes

Try it out and see how much smoother the navigation feels!