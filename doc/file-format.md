# Lego file format

File extensions.

- Minstorms: `lms`
- Spike: `llsp3`

It's a zip file with these entries:

- `manifest.json` - metadata about the file.
    - `type` can be "word-blocks", "icon-blocks", "python".
    - `name` is the display name of the project.
    - dates.
    - some other UI state.
- `icon.svg`
- `monitors.json` (Python projects, when `hasMonitors` is true in manifest)
- `projectbody.json` (Python projects)
    - One key, `main`, whose value is the Python source.
- `scratch.sb3` (Icon or Word block projects, see below)

## Scratch file format

[Official docs](https://en.scratch-wiki.info/wiki/Scratch_File_Format).

File contents:

- `project.json` - the scratch program.
- `*.wav` - custom sounds.
- `dead*.svg` - not sure, these are empty for me.
