# Mind meld (in Rust) (July 2025 ideas)

The goal is still to do these two things:

1. Track changes to mindstorms projects on a single computer.
2. Share changes between computers.

## Sketch

For tracking changes:

```
# Show which files exist, which are tracked, which stores are used.
$ mm

# Create a Git repository where changes will be tracked.
# Maybe add support for jj, loro, pijul, darcs, etc.
$ mm store create --git path/to/repo
$ mm store remove path/to/repo

# Add a file to track.
$ mm track --spike "Project 1.llsp3"
$ mm track --mindstorms "Project 1.lms"

# Copy changes from working copy to version control.
$ mm commit

# Continuously add changes to version control.
$ mm watch
```

Later:
* Add a GUI.
* Add sync across computers. Need to sort out how conflicts will work.
