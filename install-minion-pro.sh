#!/bin/bash
set -e

OS="$(uname -s)"

# ── Locate groff font directory ────────────────────────────────────────────────
echo "checking for groff..."
if ! command -v groff &>/dev/null; then
  echo "groff not found." 1>&2
  if [[ "$OS" == "Darwin" ]]; then
    echo "Install with: brew install groff" 1>&2
  else
    echo "Install with: sudo apt install groff  OR  sudo pacman -S groff" 1>&2
  fi
  exit 1
fi
echo "found: $(groff --version | head -1)"

echo "locating groff font directory..."
if [[ "$OS" == "Darwin" ]]; then
  GROFF_CELLAR=$(brew ls groff 2>/dev/null | grep "bin/groff" | sed 's;/bin/groff;;')
  if [[ -z "$GROFF_CELLAR" ]]; then
    echo "could not locate groff cellar!" 1>&2
    exit 1
  fi
  GROFF_FONT_DIR="$GROFF_CELLAR/share/groff/current/font/devpdf"
else
  # System devpdf dir (for enc/, map/, download)
  GROFF_SYSTEM_DEVPDF=$(find /usr/share/groff -maxdepth 4 -type d -name devpdf 2>/dev/null | head -1)
  if [[ -z "$GROFF_SYSTEM_DEVPDF" ]]; then
    echo "could not locate groff devpdf font directory under /usr/share/groff!" 1>&2
    exit 1
  fi
  # Install into site-font so system groff upgrades don't wipe our fonts
  GROFF_FONT_DIR="/usr/local/share/groff/site-font/devpdf"
  mkdir -p "$GROFF_FONT_DIR"
fi
echo "found system devpdf: $GROFF_SYSTEM_DEVPDF"
echo "install target:      $GROFF_FONT_DIR"

if [[ "$OS" == "Darwin" ]]; then
  _REF_DIR="$GROFF_FONT_DIR"
else
  _REF_DIR="$GROFF_SYSTEM_DEVPDF"
fi
ENC="$_REF_DIR/enc/text.enc"
TEXTMAP="$_REF_DIR/map/text.map"
DOWNLOAD="$GROFF_FONT_DIR/download"

# ── Check dependencies ─────────────────────────────────────────────────────────
for tool in fontforge afmtodit; do
  if ! command -v "$tool" &>/dev/null; then
    echo "$tool not found." 1>&2
    if [[ "$OS" == "Darwin" ]]; then
      echo "Install with: brew install $tool" 1>&2
    else
      echo "Install with: sudo apt install $tool  OR  sudo pacman -S $tool" 1>&2
    fi
    exit 1
  fi
done

# ── Locate Minion Pro OTF files ────────────────────────────────────────────────
echo "searching for Minion Pro OTF fonts..."
declare -A VARIANTS  # maps groff-name -> otf-path

if [[ "$OS" == "Darwin" ]]; then
  FONT_SEARCH_DIRS=(
    "$HOME/Library/Fonts"
    "/Library/Fonts"
    "$HOME/Library/Application Support/Adobe/Fonts"
    "/Library/Application Support/Adobe/Fonts"
    "$HOME/Downloads/Dev-Tools/minion-pro"
  )
else
  FONT_SEARCH_DIRS=(
    "$HOME/.fonts"
    "$HOME/.local/share/fonts"
    "/usr/share/fonts"
    "/usr/local/share/fonts"
  )
fi

for dir in "${FONT_SEARCH_DIRS[@]}"; do
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
  echo "No Minion Pro OTF files found on this system."
  echo ""
  read -r -p "Download Minion Pro from font.download now? [y/N] " answer
  if [[ ! "$answer" =~ ^[Yy]$ ]]; then
    echo "Aborted." 1>&2
    exit 1
  fi

  if [[ "$OS" == "Darwin" ]]; then
    INSTALL_FONT_DIR="$HOME/Library/Fonts"
  else
    INSTALL_FONT_DIR="$HOME/.local/share/fonts"
  fi
  mkdir -p "$INSTALL_FONT_DIR"

  ZIP=$(mktemp --suffix=.zip)
  trap 'rm -f "$ZIP"' RETURN
  echo "downloading minion-pro.zip..."
  curl -fL --progress-bar "https://font.download/dl/font/minion-pro.zip" -o "$ZIP"
  echo "extracting OTF files to $INSTALL_FONT_DIR..."
  unzip -o -j "$ZIP" "*.otf" -d "$INSTALL_FONT_DIR"
  [[ "$OS" != "Darwin" ]] && fc-cache -f "$INSTALL_FONT_DIR"

  # Re-scan now that fonts are installed
  for dir in "${FONT_SEARCH_DIRS[@]}"; do
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
    echo "Download succeeded but no MinionPro-*.otf files found in the zip." 1>&2
    exit 1
  fi
fi

echo "found ${#VARIANTS[@]} variant(s): ${!VARIANTS[*]}"

# Warn about missing variants used by mom
for needed in MinionR MinionI MinionB MinionBI; do
  if [[ -z "${VARIANTS[$needed]+x}" ]]; then
    echo "warning: $needed not found (mom may fall back to a substitute)"
  fi
done

# ── Generate AFM and groff font files ─────────────────────────────────────────
WORK_DIR=$(mktemp -d)
trap 'rm -rf "$WORK_DIR"' EXIT

# sed -i behaves differently on macOS vs GNU
sedi() {
  if [[ "$OS" == "Darwin" ]]; then
    sed -i '' "$@"
  else
    sed -i "$@"
  fi
}

for groff_name in "${!VARIANTS[@]}"; do
  otf="${VARIANTS[$groff_name]}"
  afm="$WORK_DIR/$groff_name.afm"
  pfa="$GROFF_FONT_DIR/$groff_name.pfa"
  dest="$GROFF_FONT_DIR/$groff_name"

  echo "processing $groff_name ($otf)..."

  # Extract AFM metrics and generate PFA from OTF
  fontforge -lang=ff -c "Open(\"$otf\"); Generate(\"$afm\"); Generate(\"$pfa\");" 2>/dev/null

  if [[ ! -f "$afm" ]]; then
    echo "  failed to generate AFM for $groff_name, skipping" 1>&2
    continue
  fi
  if [[ ! -f "$pfa" ]]; then
    echo "  failed to generate PFA for $groff_name, skipping" 1>&2
    continue
  fi
  echo "  installed $pfa"

  # Get PostScript font name from AFM
  ps_name=$(grep "^FontName" "$afm" | awk '{print $2}')

  # Create groff font description file
  case "$groff_name" in
    *I|*BI) afmtodit_opts="-i50" ;;
    *)      afmtodit_opts="-i0 -m" ;;
  esac
  # shellcheck disable=SC2086
  afmtodit -e "$ENC" $afmtodit_opts "$afm" "$TEXTMAP" "$dest"

  # Inject correct groff name into the font file
  sedi "s/^name .*/name $groff_name/" "$dest"

  echo "  installed $dest"

  # Add or update entry in download file (must point to PFA, not OTF)
  if grep -q "^	$ps_name	" "$DOWNLOAD" 2>/dev/null; then
    sedi "s|^	$ps_name	.*|	$ps_name	$pfa|" "$DOWNLOAD"
    echo "  updated download entry for $ps_name -> $pfa"
  else
    printf '\t%s\t%s\n' "$ps_name" "$pfa" >> "$DOWNLOAD"
    echo "  added download entry for $ps_name -> $pfa"
  fi
done

echo "done. Installed: ${!VARIANTS[*]}"
