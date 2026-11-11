#!/usr/bin/env bun
#MISE description="Seal the PDA10 font for the repo with ZAKA_PDA_FONT_SECRET"
#MISE dir="{{config_root}}/website/zaka_web_ui_common/assets/fonts"

echo $ZAKA_PDA_FONT_SECRET | age-keygen -y \
  | age -R - -o PDA10BurmeseMono-Regular.ttf.age PDA10BurmeseMono-Regular.ttf
