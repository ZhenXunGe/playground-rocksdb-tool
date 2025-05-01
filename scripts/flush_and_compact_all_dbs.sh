#!/bin/bash

workspace=$(realpath "$1")

md5_list=$(find "$workspace"/images/*/*/* -type d -print | xargs -n1 basename | paste -sd ' ')

cargo run --release flush-and-compact --workspace "$workspace" --md5-list $md5_list
