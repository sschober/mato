#!/usr/bin/env lua
local file_helper = require "file_helper"
local wt_cli = require "wt_cli"

if( #arg < 1 ) then
  error("expected file name as first arg")
end

local file_to_open=arg[1]

file_helper.create_if_not_exists(file_to_open)

local pdf_file_name = file_helper.replace_extension(file_to_open, ".pdf")
print("pdf_file_name: " .. pdf_file_name)

local micro_pane_id = wt_cli.spawn("micro "..file_to_open)

local matopdf_cmd = "$HOME/.cargo/bin/matopdf -w -v " .. file_to_open
local matopdf_pane_id = wt_cli.split_pane_id(micro_pane_id,
  " --percent 10 --bottom ", matopdf_cmd )


-- we need to wait for matopdf to finish, otherwise
-- termpdf.py bails out, because the pdf file does
-- not exist
os.execute( "sleep 1" )

local termpdf_cmd = "$HOME/bin/termpdf.py " .. pdf_file_name
local termpdf_pane_id = wt_cli.split_pane_id(micro_pane_id,
  " --top-level --right ", termpdf_cmd )

wt_cli.activate_pane(micro_pane_id)
