#!/bin/bash
set -e

OS="$(uname -s)"

REINSTALL=0
for arg in "$@"; do
  case "$arg" in
    --reinstall) REINSTALL=1 ;;
    *) echo "unknown argument: $arg" 1>&2; exit 1 ;;
  esac
done

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

if [[ ! -w "$GROFF_FONT_DIR" ]]; then
  echo "no write permission for $GROFF_FONT_DIR — re-run with sudo" 1>&2
  exit 1
fi

if [[ "$OS" == "Darwin" ]]; then
  _REF_DIR="$GROFF_FONT_DIR"
else
  _REF_DIR="$GROFF_SYSTEM_DEVPDF"
fi
ENC="$_REF_DIR/enc/text.enc"
TEXTMAP="$_REF_DIR/map/text.map"
DOWNLOAD="$GROFF_FONT_DIR/download"

# ── Check dependencies ─────────────────────────────────────────────────────────
for tool in fontforge afmtodit fc-query; do
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

# Build an extended map file: system text.map + self-referential mappings for
# .oldstyle and .taboldstyle glyphs (not present in the default text.map).
EXTENDED_TEXTMAP="$WORK_DIR/text.map"
{
  cat "$TEXTMAP"
  for suffix in oldstyle taboldstyle; do
    for base in zero one two three four five six seven eight nine \
                cent dollar Euro franc lira sterling yen \
                colonmonetary florin franc numbersign percent perthousand \
                estimated; do
      echo "${base}.${suffix} ${base}.${suffix}"
    done
  done
} > "$EXTENDED_TEXTMAP"

# ── Install a font variant into the groff devpdf directory ─────────────────────
# Usage: install_font_variant <groff-name> <font-spec> <afmtodit-opts>
#   font-spec is a path accepted by fontforge: plain OTF/TTF, or file.ttc(N)
#   for a specific face within a TrueType Collection.
#   afmtodit options: serif italics use -i50; monospace variants use -i0 -m.
install_font_variant() {
  local groff_name="$1"
  local font_spec="$2"
  local afmtodit_opts="$3"

  local afm="$WORK_DIR/$groff_name.afm"
  local work_pfa="$WORK_DIR/$groff_name.pfa"
  local pfa="$GROFF_FONT_DIR/$groff_name.pfa"
  local dest="$GROFF_FONT_DIR/$groff_name"

  if [[ -f "$dest" ]]; then
    if [[ "$REINSTALL" -eq 0 ]]; then
      echo "  $groff_name already installed, skipping (use --reinstall to force)"
      return 0
    fi
    echo "  $groff_name already installed, reinstalling..."
    rm -f "$dest" "$pfa"
  fi

  echo "  processing $groff_name ($font_spec)..."

  # Generate into WORK_DIR so fontforge never needs to write to the protected
  # groff font directory.  Use the Python API: handles plain files and TTC specs.
  fontforge -lang=py -c "
import fontforge
f = fontforge.open('$font_spec')
f.generate('$afm')
f.generate('$work_pfa')
" 2>/dev/null || true

  if [[ ! -f "$afm" ]]; then
    echo "    failed to generate AFM for $groff_name, skipping" 1>&2
    return 1
  fi
  if [[ ! -f "$work_pfa" ]]; then
    echo "    failed to generate PFA for $groff_name, skipping" 1>&2
    return 1
  fi
  cp "$work_pfa" "$pfa"
  echo "    installed $pfa"

  local ps_name
  ps_name=$(grep "^FontName" "$afm" | awk '{print $2}')

  # shellcheck disable=SC2086
  afmtodit -e "$ENC" $afmtodit_opts "$afm" "$EXTENDED_TEXTMAP" "$dest"
  sedi "s/^name .*/name $groff_name/" "$dest"
  echo "    installed $dest"

  if grep -q "^	$ps_name	" "$DOWNLOAD" 2>/dev/null; then
    sedi "s|^	$ps_name	.*|	$ps_name	$pfa|" "$DOWNLOAD"
    echo "    updated download entry for $ps_name -> $pfa"
  else
    printf '\t%s\t%s\n' "$ps_name" "$pfa" >> "$DOWNLOAD"
    echo "    added download entry for $ps_name -> $pfa"
  fi
}

# Returns the face index within a TTC file that matches the given PostScript
# name, by parsing fc-query output.  Prints the index, or nothing on failure.
_ttc_index_for_psname() {
  local file="$1"
  local psname="$2"
  fc-query "$file" 2>/dev/null | awk -v ps="$psname" '
    /[[:space:]]index:/ { match($0, /[0-9]+/); idx = substr($0, RSTART, RLENGTH) }
    /[[:space:]]postscriptname:/ {
      gsub(/.*"/, ""); gsub(/".*/, "")
      if ($0 == ps) { print idx; exit }
    }
  '
}

# ── Minion Pro ─────────────────────────────────────────────────────────────────
echo ""
echo "── Minion Pro ────────────────────────────────────────────────────────────────"
echo "searching for Minion Pro OTF fonts..."
declare -A MINION_VARIANTS  # maps groff-name -> otf-path

if [[ "$OS" == "Darwin" ]]; then
  MINION_SEARCH_DIRS=(
    "$HOME/Library/Fonts"
    "/Library/Fonts"
    "$HOME/Library/Application Support/Adobe/Fonts"
    "/Library/Application Support/Adobe/Fonts"
    "$HOME/Downloads/Dev-Tools/minion-pro"
  )
else
  MINION_SEARCH_DIRS=(
    "$HOME/.fonts"
    "$HOME/.local/share/fonts"
    "/usr/share/fonts"
    "/usr/local/share/fonts"
  )
fi

_scan_minion() {
  for dir in "${MINION_SEARCH_DIRS[@]}"; do
    [[ -d "$dir" ]] || continue
    while IFS= read -r otf; do
      local base
      base=$(basename "$otf" .otf)
      case "$base" in
        MinionPro-Regular)  MINION_VARIANTS[MinionR]="$otf" ;;
        MinionPro-Bold)     MINION_VARIANTS[MinionB]="$otf" ;;
        MinionPro-It)       MINION_VARIANTS[MinionI]="$otf" ;;
        MinionPro-BoldIt)   MINION_VARIANTS[MinionBI]="$otf" ;;
      esac
    done < <(find "$dir" -name "MinionPro-*.otf" 2>/dev/null)
  done
}

_scan_minion

if [[ ${#MINION_VARIANTS[@]} -eq 0 ]]; then
  echo "No Minion Pro OTF files found on this system."
  echo ""
  read -r -p "Download Minion Pro from font.download now? [y/N] " answer
  if [[ ! "$answer" =~ ^[Yy]$ ]]; then
    echo "Skipping Minion Pro." 1>&2
  else
    if [[ "$OS" == "Darwin" ]]; then
      INSTALL_FONT_DIR="$HOME/Library/Fonts"
    else
      INSTALL_FONT_DIR="$HOME/.local/share/fonts"
    fi
    mkdir -p "$INSTALL_FONT_DIR"

    ZIP=$(mktemp --suffix=.zip)
    echo "downloading minion-pro.zip..."
    curl -fL --progress-bar "https://font.download/dl/font/minion-pro.zip" -o "$ZIP"
    echo "extracting OTF files to $INSTALL_FONT_DIR..."
    unzip -o -j "$ZIP" "*.otf" -d "$INSTALL_FONT_DIR"
    rm -f "$ZIP"
    [[ "$OS" != "Darwin" ]] && fc-cache -f "$INSTALL_FONT_DIR"

    _scan_minion

    if [[ ${#MINION_VARIANTS[@]} -eq 0 ]]; then
      echo "Download succeeded but no MinionPro-*.otf files found in the zip." 1>&2
    fi
  fi
fi

if [[ ${#MINION_VARIANTS[@]} -gt 0 ]]; then
  echo "found ${#MINION_VARIANTS[@]} variant(s): ${!MINION_VARIANTS[*]}"
  for needed in MinionR MinionI MinionB MinionBI; do
    if [[ -z "${MINION_VARIANTS[$needed]+x}" ]]; then
      echo "warning: $needed not found (mom may fall back to a substitute)"
    fi
  done
  for groff_name in "${!MINION_VARIANTS[@]}"; do
    case "$groff_name" in
      *I|*BI) opts="-i50" ;;
      *)      opts="-i0 -m" ;;
    esac
    install_font_variant "$groff_name" "${MINION_VARIANTS[$groff_name]}" "$opts"
  done
  echo "Minion Pro done. Installed: ${!MINION_VARIANTS[*]}"
else
  echo "Minion Pro: skipped (no fonts found or download declined)."
fi

# ── Iosevka Curly Slab ─────────────────────────────────────────────────────────
echo ""
echo "── Iosevka Curly Slab ────────────────────────────────────────────────────────"
echo "searching for IosevkaCurlySlab font files..."
declare -A IOSEVKA_VARIANTS  # maps groff-name -> font file path

if [[ "$OS" == "Darwin" ]]; then
  IOSEVKA_SEARCH_DIRS=(
    "$HOME/Library/Fonts"
    "/Library/Fonts"
  )
else
  IOSEVKA_SEARCH_DIRS=(
    "$HOME/.fonts"
    "$HOME/.local/share/fonts"
    "/usr/share/fonts"
    "/usr/local/share/fonts"
  )
fi

_scan_iosevka() {
  for dir in "${IOSEVKA_SEARCH_DIRS[@]}"; do
    [[ -d "$dir" ]] || continue
    local f

    # Prefer individual .ttf files (one face per file, no index needed).
    # Fall back to .ttc collections resolved via fc-query if .ttf is absent.
    f=$(find "$dir" -name "IosevkaCurlySlab-Regular.ttf" 2>/dev/null | head -1)
    if [[ -n "$f" ]]; then IOSEVKA_VARIANTS[IosevkaCurlySlabR]="$f"; fi

    f=$(find "$dir" -name "IosevkaCurlySlab-Italic.ttf" 2>/dev/null | head -1)
    if [[ -n "$f" ]]; then IOSEVKA_VARIANTS[IosevkaCurlySlabI]="$f"; fi

    f=$(find "$dir" -name "IosevkaCurlySlab-Bold.ttf" 2>/dev/null | head -1)
    if [[ -n "$f" ]]; then IOSEVKA_VARIANTS[IosevkaCurlySlabB]="$f"; fi

    f=$(find "$dir" -name "IosevkaCurlySlab-BoldItalic.ttf" 2>/dev/null | head -1)
    if [[ -n "$f" ]]; then IOSEVKA_VARIANTS[IosevkaCurlySlabBI]="$f"; fi

    if [[ ${#IOSEVKA_VARIANTS[@]} -gt 0 ]]; then
      break
    fi

    # TTC fallback: extract faces by PostScript name
    local reg_ttc bold_ttc idx
    reg_ttc=$(find "$dir" -name "IosevkaCurlySlab-Regular.ttc" 2>/dev/null | head -1)
    bold_ttc=$(find "$dir" -name "IosevkaCurlySlab-Bold.ttc" 2>/dev/null | head -1)

    if [[ -n "$reg_ttc" ]]; then
      idx=$(_ttc_index_for_psname "$reg_ttc" "Iosevka-Curly-Slab")
      if [[ -n "$idx" ]]; then IOSEVKA_VARIANTS[IosevkaCurlySlabR]="${reg_ttc}(${idx})"; fi
      idx=$(_ttc_index_for_psname "$reg_ttc" "Iosevka-Curly-Slab-Italic")
      if [[ -n "$idx" ]]; then IOSEVKA_VARIANTS[IosevkaCurlySlabI]="${reg_ttc}(${idx})"; fi
    fi

    if [[ -n "$bold_ttc" ]]; then
      idx=$(_ttc_index_for_psname "$bold_ttc" "Iosevka-Curly-Slab-Bold")
      if [[ -n "$idx" ]]; then IOSEVKA_VARIANTS[IosevkaCurlySlabB]="${bold_ttc}(${idx})"; fi
      idx=$(_ttc_index_for_psname "$bold_ttc" "Iosevka-Curly-Slab-Bold-Italic")
      if [[ -n "$idx" ]]; then IOSEVKA_VARIANTS[IosevkaCurlySlabBI]="${bold_ttc}(${idx})"; fi
    fi

    if [[ ${#IOSEVKA_VARIANTS[@]} -gt 0 ]]; then
      break
    fi
  done
}

_scan_iosevka

if [[ ${#IOSEVKA_VARIANTS[@]} -gt 0 ]]; then
  echo "found ${#IOSEVKA_VARIANTS[@]} variant(s): ${!IOSEVKA_VARIANTS[*]}"
  for needed in IosevkaCurlySlabR IosevkaCurlySlabI IosevkaCurlySlabB IosevkaCurlySlabBI; do
    if [[ -z "${IOSEVKA_VARIANTS[$needed]+x}" ]]; then
      echo "warning: $needed not found (mom may fall back to a substitute)"
    fi
  done
  # Monospace font: no italic correction, no lig/kern data for any variant
  for groff_name in "${!IOSEVKA_VARIANTS[@]}"; do
    install_font_variant "$groff_name" "${IOSEVKA_VARIANTS[$groff_name]}" "-i0 -m"
  done
  echo "Iosevka Curly Slab done. Installed: ${!IOSEVKA_VARIANTS[*]}"
else
  echo "IosevkaCurlySlab-Regular and IosevkaCurlySlab-Bold font files not found." 1>&2
  echo "Install the package (e.g. sudo pacman -S ttf-iosevka-curly-slab) and re-run." 1>&2
fi
