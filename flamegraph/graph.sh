#!/bin/sh
cat flame.folded  | inferno-flamegraph > profile.svg
firefox profile.svg
