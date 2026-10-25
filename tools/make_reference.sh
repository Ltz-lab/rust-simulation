#!/usr/bin/env bash
# Regenerate the reference observations in reference/ from SimulationCraft.
# SimC is only used as a stand-in for real combat logs: we run it, read its
# report, and never use its source code.
#
#   tools/make_reference.sh <prefix> <profile.simc> <primary-stat> [/path/to/simc-binary]
#
#   tools/make_reference.sh ret  $SIMC/profiles/MID2/MID2_Paladin_Retribution.simc strength  $SIMC/build/simc
#   tools/make_reference.sh fire $SIMC/profiles/MID2/MID2_Mage_Fire.simc         intellect $SIMC/build/simc
#
# Writes reference/<prefix>_core_observations.json plus stat probes
# (+500 crit/haste/mastery/vers rating, +200 primary stat).
set -euo pipefail
PREFIX=${1:?reference prefix, e.g. ret or fire}
PROFILE=${2:?path to a SimC profile}
PRIMARY=${3:?primary stat: strength, agility or intellect}
SIMC_BIN=${4:-$(dirname "$PROFILE")/../../build/simc}
ITERS=${ITERS:-5000}
HERE=$(cd "$(dirname "$0")" && pwd)
OUT=$HERE/../reference
WORK=$(mktemp -d)
CORE=$WORK/${PREFIX}_core.simc

# Core-kit profile: the sample profile minus trinkets, consumables, set
# bonuses and Bloodlust.
grep -v -E '^(trinket[12]|potion|flask|food|augmentation|temporary_enchant)=' "$PROFILE" > "$CORE"
cat >> "$CORE" <<'EOF'
potion=disabled
flask=disabled
food=disabled
augmentation=disabled
temporary_enchant=disabled
set_bonus=midnight_season_2_2pc=0/midnight_season_2_4pc=0/bite_of_zuljan_2pc=0
override.bloodlust=0
EOF

run() { # name extra-args...
  local name=$1; shift
  "$SIMC_BIN" "$CORE" iterations="$ITERS" threads="$(getconf _NPROCESSORS_ONLN)" json2="$WORK/$name.json" "$@" > /dev/null
  python3 -I "$HERE/observe.py" "$WORK/$name.json" > "$OUT/$name.json"
  echo "wrote reference/$name.json"
}

run "${PREFIX}_core_observations"
run "${PREFIX}_probe_mastery_rating" enchant_mastery_rating=500
run "${PREFIX}_probe_crit_rating" enchant_crit_rating=500
run "${PREFIX}_probe_haste_rating" enchant_haste_rating=500
run "${PREFIX}_probe_versatility_rating" enchant_versatility_rating=500
run "${PREFIX}_probe_${PRIMARY}" "enchant_${PRIMARY}=200"

# A single fully logged fight, for reading mechanics by eye.
"$SIMC_BIN" "$CORE" iterations=1 threads=1 log=1 vary_combat_length=0 seed=7 output="$WORK/${PREFIX}_log.txt" > /dev/null
echo "combat log for inspection: $WORK/${PREFIX}_log.txt"
