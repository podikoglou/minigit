#!/bin/bash
#
# Re-encodes an object and diffs it with the original (decompressed).
# Useful for debugging both parsing and writing.
#
# Example:
#   scripts/diff.sh ./objects/commit-4.obj
#

diff --color --text -y <(cat "$1" | pigz -zd) <(minigit parse-object -b "$1" | pigz -zd) 
