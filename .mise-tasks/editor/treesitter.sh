#!/bin/sh
#MISE description="Write the tree-sitter grammar from the lexer, then its parser, and test it"
#MISE dir="{{config_root}}"
set -e

ZAKA_BLESS=1 cargo nextest run -p zakac_lexer treesitter
cd editors/treesitter
tree-sitter generate src/grammar.json
tree-sitter test
