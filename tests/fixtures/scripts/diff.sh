#!/bin/bash
#
# Re-encodes an object and diffs it with the original (decompressed).
# Useful for debugging both parsing and writing.
#
# Respects the `$DIFF` variable (uses `diff --text --color -y` by default). Some other options:
# - vimdiff
# - nvim -d
#
# Example:
#   scripts/diff.sh ./objects/commit-4.obj
#

${DIFF:-diff --text --color -y} <(cat "$1" | pigz -zd) <(minigit parse-object -b "$1" | pigz -zd) 
