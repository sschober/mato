#!/usr/bin/env bash
#
diary_dir=$HOME/src/writing/diary
pushd $diary_dir
filename=$(date "+%Y-%m-%d")
if [[ -n $1 ]]; then
	filename=$1
fi
today=$(LANG=de_DE.UTF-8 date "+%d. %B %Y")
if [[ ! -f "$filename.md" ]]; then
	echo "# $today" >$filename.md
fi
matoedit $filename.md
popd
#kill $matogro_pid
# kitty @ close-window --match 'title:^matogro-viewer'
#wezterm cli kill-pane --pane-id $wt_pane
