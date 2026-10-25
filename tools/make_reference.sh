#!/usr/bin/env bash
# Regenerate the reference observations in reference/ from SimulationCraft.
# SimC is only used as a stand-in for real combat logs: we run it, read its
# report, and never use its source code.
#
#   tools/make_reference.sh /path/to/simc-repo [/path/to/simc-binary]
#
set -euo pipefail
SIMC_REPO=${1:?path to a simulationcraft checkout}
SIMC_BIN=${2:-$SIMC_REPO/build/simc}
HERE=$(cd "$(dirname "$0")" && pwd)
OUT=$HERE/../reference
WORK=$(mktemp -d)

# Core-kit profile: the Midnight S2 sample Ret profile minus trinkets,
# consumables, set bonuses and Bloodlust.
grep -v -E '^(trinket[12]|potion|flask|food|augmentation|temporary_enchant)=' \
  "$SIMC_REPO/profiles/MID2/MID2_Paladin_Retribution.simc" > "$WORK/ret_core.simc"
cat >> "$WORK/ret_core.simc" <<'EOF'
potion=disabled
flask=disabled
food=disabled
augmentation=disabled
temporary_enchant=disabled
set_bonus=midnight_season_2_2pc=0
set_bonus=midnight_season_2_4pc=0
set_bonus=bite_of_zuljan_2pc=0
override.bloodlust=0
EOF

run() { # name extra-args...
  local name=$1; shift
  "$SIMC_BIN" "$WORK/ret_core.simc" iterations=5000 threads="$(nproc)" json2="$WORK/$name.json" "$@" > /dev/null
  python3 -I "$HERE/observe.py" "$WORK/$name.json" > "$OUT/$name.json"
  echo "wrote reference/$name.json"
}

run ret_core_observations
run ret_probe_mastery_rating enchant_mastery_rating=500
run ret_probe_crit_rating enchant_crit_rating=500
run ret_probe_haste_rating enchant_haste_rating=500
run ret_probe_versatility_rating enchant_versatility_rating=500
run ret_probe_strength enchant_strength=200

# A single fully logged fight, for reading mechanics by eye.
"$SIMC_BIN" "$WORK/ret_core.simc" iterations=1 threads=1 log=1 vary_combat_length=0 seed=7 output="$WORK/ret_log.txt" > /dev/null
echo "combat log for inspection: $WORK/ret_log.txt"
