#!/bin/bash
set -e

# Find groff Cellar location
echo "checking for groff..."
GROFF_VERSION_HOMEBREW=$(brew list --versions | grep groff)
if [[ -z "$GROFF_VERSION_HOMEBREW" ]]; then
  echo "groff not installed via homebrew" 1>&2
  exit 1
fi
echo "found: $GROFF_VERSION_HOMEBREW"

echo "checking for groff homebrew Cellar..."
GROFF_CELLAR=$(brew ls groff | grep "bin/groff" | sed 's;/bin/groff;;')
if [[ -z "$GROFF_CELLAR" ]]; then
  echo "could not locate groff cellar!" 1>&2
  exit 1
fi
echo "found: $GROFF_CELLAR"

GROFF_FONT_DIR="$GROFF_CELLAR/share/groff/current/font/devpdf"
ENC="$GROFF_FONT_DIR/enc/text.enc"
TEXTMAP="$GROFF_FONT_DIR/map/text.map"
DOWNLOAD="$GROFF_FONT_DIR/download"

# Check dependencies
for tool in fontforge afmtodit; do
  if ! command -v "$tool" &>/dev/null; then
    echo "$tool not found. Install with: brew install ${tool}" 1>&2
    exit 1
  fi
done

# Locate Minion Pro OTF files on system
echo "searching for Minion Pro OTF fonts..."
declare -A VARIANTS  # maps groff-name -> otf-path

for dir in "$HOME/Library/Fonts" "/Library/Fonts" \
           "$HOME/Library/Application Support/Adobe/Fonts" \
           "/Library/Application Support/Adobe/Fonts" \
           "$HOME/Downloads/Dev-Tools/minion-pro"; do
  [[ -d "$dir" ]] || continue
  while IFS= read -r otf; do
    base=$(basename "$otf" .otf)
    case "$base" in
      MinionPro-Regular)  VARIANTS[MinionR]="$otf" ;;
      MinionPro-Bold)     VARIANTS[MinionB]="$otf" ;;
      MinionPro-It)       VARIANTS[MinionI]="$otf" ;;
      MinionPro-BoldIt)   VARIANTS[MinionBI]="$otf" ;;
    esac
  done < <(find "$dir" -name "MinionPro-*.otf" 2>/dev/null)
done

if [[ ${#VARIANTS[@]} -eq 0 ]]; then
  echo "no Minion Pro OTF files found on system" 1>&2
  exit 1
fi

echo "found ${#VARIANTS[@]} variant(s): ${!VARIANTS[*]}"

# Warn about missing variants used by mom
for needed in MinionR MinionI MinionB MinionBI; do
  if [[ -z "${VARIANTS[$needed]+x}" ]]; then
    echo "warning: $needed not found (mom may fall back to a substitute)"
  fi
done

# Generate AFM and groff font files, update download
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

for groff_name in "${!VARIANTS[@]}"; do
  otf="${VARIANTS[$groff_name]}"
  ps_name=$(basename "$otf" .otf | tr '-' '_' | sed 's/_/-/g')  # recover original name
  afm="$TMPDIR/$groff_name.afm"
  dest="$GROFF_FONT_DIR/$groff_name"

  echo "processing $groff_name ($otf)..."

  # Extract AFM metrics from OTF
  fontforge -lang=ff -c "Open(\"$otf\"); Generate(\"$afm\");" 2>/dev/null

  if [[ ! -f "$afm" ]]; then
    echo "  failed to generate AFM for $groff_name, skipping" 1>&2
    continue
  fi

  # Get PostScript font name from AFM
  ps_name=$(grep "^FontName" "$afm" | awk '{print $2}')

  # Create groff font description file
  # -e: encoding file, second positional arg: PS-name-to-groff-name map
  # -i50 for italic fonts, -i0 -m for upright fonts
  case "$groff_name" in
    *I|*BI) afmtodit_opts="-i50" ;;
    *)      afmtodit_opts="-i0 -m" ;;
  esac
  # shellcheck disable=SC2086
  afmtodit -e "$ENC" $afmtodit_opts "$afm" "$TEXTMAP" "$dest"

  # Inject correct groff name into the font file
  sed -i '' "s/^name .*/name $groff_name/" "$dest"

  echo "  installed $dest"

  # Add or update entry in download file
  if grep -q "^	$ps_name	" "$DOWNLOAD" 2>/dev/null; then
    # Update existing entry
    sed -i '' "s|^	$ps_name	.*|	$ps_name	$otf|" "$DOWNLOAD"
    echo "  updated download entry for $ps_name"
  else
    printf '\t%s\t%s\n' "$ps_name" "$otf" >> "$DOWNLOAD"
    echo "  added download entry for $ps_name"
  fi
done

echo "done. Installed: ${!VARIANTS[*]}"
