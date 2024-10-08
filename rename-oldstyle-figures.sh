#!/usr/bin/env bash

font_file=$1

echo "warning: this is not idempotent!"

for fig in zero one two three four five six seven eight nine; do
  echo $fig
  gsed -i "s;^\([-]*\)\(.*\)\($fig.oldstyle\);\3\2\3;" $font_file
done
