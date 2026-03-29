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
  # Derive site-font from the groff binary prefix (e.g. /opt/homebrew/bin/groff -> /opt/homebrew/etc/groff/site-font)
  GROFF_BIN=$(command -v groff)
  GROFF_PREFIX=$(dirname "$(dirname "$GROFF_BIN")")
  GROFF_SITE_FONT="$GROFF_PREFIX/etc/groff/site-font"
  if [[ ! -d "$GROFF_SITE_FONT" && ! -d "$(dirname "$GROFF_SITE_FONT")" ]]; then
    echo "could not determine groff site-font directory (expected $GROFF_SITE_FONT)" 1>&2
    exit 1
  fi
  GROFF_FONT_DIR="$GROFF_SITE_FONT/devpdf"
  mkdir -p "$GROFF_FONT_DIR"
  # Find the system devpdf for enc/ and map/ references
  GROFF_CELLAR=$(brew ls groff 2>/dev/null | grep "bin/groff" | sed 's;/bin/groff;;')
  if [[ -n "$GROFF_CELLAR" ]]; then
    GROFF_SYSTEM_DEVPDF="$GROFF_CELLAR/share/groff/current/font/devpdf"
  else
    GROFF_SYSTEM_DEVPDF=$(find "$(brew --prefix groff 2>/dev/null)/share/groff" -maxdepth 4 -type d -name devpdf 2>/dev/null | head -1)
  fi
  if [[ -z "$GROFF_SYSTEM_DEVPDF" || ! -d "$GROFF_SYSTEM_DEVPDF" ]]; then
    echo "could not locate groff system devpdf directory!" 1>&2
    exit 1
  fi
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

_REF_DIR="$GROFF_SYSTEM_DEVPDF"
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

# Build an extended map file: system text.map + mappings for oldstyle numeral
# glyph names not present in the default text.map.
#
# Format: <PostScript-glyph-name> <groff-character-name>
#
# Fonts use different PostScript names for oldstyle figures:
#   .oldstyle    — Minion Pro (e.g. eight.oldstyle)
#   .taboldstyle — Minion Pro tabular variant
#   .tosf        — Alegreya and many other OFL fonts (traditional oldstyle figures)
#
# All variants are mapped to the same groff character names (e.g. eight.oldstyle)
# so mato's \[eight.oldstyle] escapes work regardless of which font is active.
EXTENDED_TEXTMAP="$WORK_DIR/text.map"
{
  cat "$TEXTMAP"
  for base in zero one two three four five six seven eight nine \
              cent dollar Euro franc lira sterling yen \
              colonmonetary florin franc numbersign percent perthousand \
              estimated; do
    echo "${base}.oldstyle    ${base}.oldstyle"
    echo "${base}.taboldstyle ${base}.taboldstyle"
    echo "${base}.tosf        ${base}.oldstyle"
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

  # Remove broken symlinks (e.g. stale pointers into an old Homebrew Cellar after
  # `brew cleanup`).  -L is true for symlinks whether or not the target exists;
  # -e is false when the target is gone.  A broken symlink causes open() to fail
  # with ENOENT even though the path itself exists in the directory listing.
  if [[ -L "$dest" && ! -e "$dest" ]]; then
    echo "  $groff_name: removing broken symlink (stale cellar reference)"
    rm -f "$dest"
  fi
  if [[ -L "$pfa" && ! -e "$pfa" ]]; then
    rm -f "$pfa"
  fi

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

  mkdir -p "$GROFF_FONT_DIR"
  # Run from $_REF_DIR so afmtodit finds DESC there; dest is an absolute path.
  # shellcheck disable=SC2086
  (cd "$_REF_DIR" && afmtodit -e "$ENC" $afmtodit_opts "$afm" "$EXTENDED_TEXTMAP" "$dest")
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
MINION_KEYS=()  # tracks which groff-names have been found (bash 3.2 compat)

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
  MINION_KEYS=()
  for dir in "${MINION_SEARCH_DIRS[@]}"; do
    [[ -d "$dir" ]] || continue
    local _before=${#MINION_KEYS[@]}
    while IFS= read -r otf; do
      local base
      base=$(basename "$otf" .otf)
      case "$base" in
        MinionPro-Regular)  MINION_MinionR="$otf";  MINION_KEYS+=(MinionR)  ;;
        MinionPro-Bold)     MINION_MinionB="$otf";  MINION_KEYS+=(MinionB)  ;;
        MinionPro-It)       MINION_MinionI="$otf";  MINION_KEYS+=(MinionI)  ;;
        MinionPro-BoldIt)   MINION_MinionBI="$otf"; MINION_KEYS+=(MinionBI) ;;
      esac
    done < <(find "$dir" -name "MinionPro-*.otf" 2>/dev/null)
    [[ ${#MINION_KEYS[@]} -gt $_before ]] && break
  done
}

_scan_minion

if [[ ${#MINION_KEYS[@]} -eq 0 ]]; then
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

    if [[ ${#MINION_KEYS[@]} -eq 0 ]]; then
      echo "Download succeeded but no MinionPro-*.otf files found in the zip." 1>&2
    fi
  fi
fi

if [[ ${#MINION_KEYS[@]} -gt 0 ]]; then
  echo "found ${#MINION_KEYS[@]} variant(s): ${MINION_KEYS[*]}"
  for needed in MinionR MinionI MinionB MinionBI; do
    _v="MINION_${needed}"
    if [[ -z "${!_v}" ]]; then
      echo "warning: $needed not found (mom may fall back to a substitute)"
    fi
  done
  for groff_name in "${MINION_KEYS[@]}"; do
    case "$groff_name" in
      *I|*BI) opts="-i50" ;;
      *)      opts="-i0 -m" ;;
    esac
    _v="MINION_${groff_name}"
    install_font_variant "$groff_name" "${!_v}" "$opts"
  done
  echo "Minion Pro done. Installed: ${MINION_KEYS[*]}"
else
  echo "Minion Pro: skipped (no fonts found or download declined)."
fi

# ── Iosevka Curly Slab ─────────────────────────────────────────────────────────
echo ""
echo "── Iosevka Curly Slab ────────────────────────────────────────────────────────"
echo "searching for IosevkaCurlySlab font files..."
IOSEVKA_KEYS=()  # tracks which groff-names have been found (bash 3.2 compat)

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
  IOSEVKA_KEYS=()
  for dir in "${IOSEVKA_SEARCH_DIRS[@]}"; do
    [[ -d "$dir" ]] || continue
    local f

    # Prefer individual .ttf files (one face per file, no index needed).
    # Fall back to .ttc collections resolved via fc-query if .ttf is absent.
    f=$(find "$dir" -name "IosevkaCurlySlab-Regular.ttf" 2>/dev/null | head -1)
    if [[ -n "$f" ]]; then IOSEVKA_IosevkaCurlySlabR="$f"; IOSEVKA_KEYS+=(IosevkaCurlySlabR); fi

    f=$(find "$dir" -name "IosevkaCurlySlab-Italic.ttf" 2>/dev/null | head -1)
    if [[ -n "$f" ]]; then IOSEVKA_IosevkaCurlySlabI="$f"; IOSEVKA_KEYS+=(IosevkaCurlySlabI); fi

    f=$(find "$dir" -name "IosevkaCurlySlab-Bold.ttf" 2>/dev/null | head -1)
    if [[ -n "$f" ]]; then IOSEVKA_IosevkaCurlySlabB="$f"; IOSEVKA_KEYS+=(IosevkaCurlySlabB); fi

    f=$(find "$dir" -name "IosevkaCurlySlab-BoldItalic.ttf" 2>/dev/null | head -1)
    if [[ -n "$f" ]]; then IOSEVKA_IosevkaCurlySlabBI="$f"; IOSEVKA_KEYS+=(IosevkaCurlySlabBI); fi

    if [[ ${#IOSEVKA_KEYS[@]} -gt 0 ]]; then
      break
    fi

    # TTC fallback: extract faces by PostScript name.
    # Handles both split collections (IosevkaCurlySlab-Regular.ttc / -Bold.ttc)
    # and a single combined collection (IosevkaCurlySlab.ttc).
    local reg_ttc bold_ttc combined_ttc idx
    reg_ttc=$(find "$dir" -name "IosevkaCurlySlab-Regular.ttc" 2>/dev/null | head -1)
    bold_ttc=$(find "$dir" -name "IosevkaCurlySlab-Bold.ttc" 2>/dev/null | head -1)
    combined_ttc=$(find "$dir" -name "IosevkaCurlySlab.ttc" 2>/dev/null | head -1)

    # Combined .ttc (e.g. IosevkaCurlySlab.ttc) — all four faces in one file
    if [[ -n "$combined_ttc" ]]; then
      idx=$(_ttc_index_for_psname "$combined_ttc" "IosevkaCurlySlab")
      if [[ -n "$idx" ]]; then IOSEVKA_IosevkaCurlySlabR="${combined_ttc}(${idx})"; IOSEVKA_KEYS+=(IosevkaCurlySlabR); fi
      idx=$(_ttc_index_for_psname "$combined_ttc" "IosevkaCurlySlab-Italic")
      if [[ -n "$idx" ]]; then IOSEVKA_IosevkaCurlySlabI="${combined_ttc}(${idx})"; IOSEVKA_KEYS+=(IosevkaCurlySlabI); fi
      idx=$(_ttc_index_for_psname "$combined_ttc" "IosevkaCurlySlab-Bold")
      if [[ -n "$idx" ]]; then IOSEVKA_IosevkaCurlySlabB="${combined_ttc}(${idx})"; IOSEVKA_KEYS+=(IosevkaCurlySlabB); fi
      idx=$(_ttc_index_for_psname "$combined_ttc" "IosevkaCurlySlab-BoldItalic")
      if [[ -n "$idx" ]]; then IOSEVKA_IosevkaCurlySlabBI="${combined_ttc}(${idx})"; IOSEVKA_KEYS+=(IosevkaCurlySlabBI); fi
    fi

    if [[ -n "$reg_ttc" ]]; then
      idx=$(_ttc_index_for_psname "$reg_ttc" "IosevkaCurlySlab")
      if [[ -n "$idx" ]]; then IOSEVKA_IosevkaCurlySlabR="${reg_ttc}(${idx})"; IOSEVKA_KEYS+=(IosevkaCurlySlabR); fi
      idx=$(_ttc_index_for_psname "$reg_ttc" "IosevkaCurlySlab-Italic")
      if [[ -n "$idx" ]]; then IOSEVKA_IosevkaCurlySlabI="${reg_ttc}(${idx})"; IOSEVKA_KEYS+=(IosevkaCurlySlabI); fi
    fi

    if [[ -n "$bold_ttc" ]]; then
      idx=$(_ttc_index_for_psname "$bold_ttc" "IosevkaCurlySlab-Bold")
      if [[ -n "$idx" ]]; then IOSEVKA_IosevkaCurlySlabB="${bold_ttc}(${idx})"; IOSEVKA_KEYS+=(IosevkaCurlySlabB); fi
      idx=$(_ttc_index_for_psname "$bold_ttc" "IosevkaCurlySlab-BoldItalic")
      if [[ -n "$idx" ]]; then IOSEVKA_IosevkaCurlySlabBI="${bold_ttc}(${idx})"; IOSEVKA_KEYS+=(IosevkaCurlySlabBI); fi
    fi

    if [[ ${#IOSEVKA_KEYS[@]} -gt 0 ]]; then
      break
    fi

    # If a TTC was found but no faces matched, print the actual PostScript names
    # to help diagnose naming mismatches.
    for _ttc in "$combined_ttc" "$reg_ttc" "$bold_ttc"; do
      [[ -z "$_ttc" ]] && continue
      echo "  found TTC but no faces matched: $_ttc"
      echo "  PostScript names in file:"
      fc-query "$_ttc" 2>/dev/null | awk '/postscriptname:/ { gsub(/.*"/, ""); gsub(/".*/, ""); print "    " $0 }'
    done
  done
}

_scan_iosevka

if [[ ${#IOSEVKA_KEYS[@]} -gt 0 ]]; then
  echo "found ${#IOSEVKA_KEYS[@]} variant(s): ${IOSEVKA_KEYS[*]}"
  for needed in IosevkaCurlySlabR IosevkaCurlySlabI IosevkaCurlySlabB IosevkaCurlySlabBI; do
    _v="IOSEVKA_${needed}"
    if [[ -z "${!_v}" ]]; then
      echo "warning: $needed not found (mom may fall back to a substitute)"
    fi
  done
  # Monospace font: no italic correction, no lig/kern data for any variant
  for groff_name in "${IOSEVKA_KEYS[@]}"; do
    _v="IOSEVKA_${groff_name}"
    install_font_variant "$groff_name" "${!_v}" "-i0 -m"
  done
  echo "Iosevka Curly Slab done. Installed: ${IOSEVKA_KEYS[*]}"
else
  echo "IosevkaCurlySlab-Regular and IosevkaCurlySlab-Bold font files not found." 1>&2
  echo "Install the package (e.g. sudo pacman -S ttf-iosevka-curly-slab) and re-run." 1>&2
fi

# ── Alegreya ───────────────────────────────────────────────────────────────────
echo ""
echo "── Alegreya ──────────────────────────────────────────────────────────────────"
echo "searching for Alegreya OTF fonts..."
ALEGREYA_KEYS=()  # tracks which groff-names have been found (bash 3.2 compat)

if [[ "$OS" == "Darwin" ]]; then
  ALEGREYA_SEARCH_DIRS=(
    "$HOME/Library/Fonts"
    "/Library/Fonts"
  )
else
  ALEGREYA_SEARCH_DIRS=(
    "$HOME/.fonts"
    "$HOME/.local/share/fonts"
    "/usr/share/fonts"
    "/usr/local/share/fonts"
  )
fi

_scan_alegreya() {
  ALEGREYA_KEYS=()
  for dir in "${ALEGREYA_SEARCH_DIRS[@]}"; do
    [[ -d "$dir" ]] || continue
    local _before=${#ALEGREYA_KEYS[@]}
    while IFS= read -r otf; do
      local base
      base=$(basename "$otf" .otf)
      case "$base" in
        Alegreya-Regular)    ALEGREYA_AlegreyaR="$otf";  ALEGREYA_KEYS+=(AlegreyaR)  ;;
        Alegreya-Bold)       ALEGREYA_AlegreyaB="$otf";  ALEGREYA_KEYS+=(AlegreyaB)  ;;
        Alegreya-Italic)     ALEGREYA_AlegreyaI="$otf";  ALEGREYA_KEYS+=(AlegreyaI)  ;;
        Alegreya-BoldItalic) ALEGREYA_AlegreyaBI="$otf"; ALEGREYA_KEYS+=(AlegreyaBI) ;;
      esac
    done < <(find "$dir" -name "Alegreya-*.otf" 2>/dev/null)
    [[ ${#ALEGREYA_KEYS[@]} -gt $_before ]] && break
  done
}

_scan_alegreya

if [[ ${#ALEGREYA_KEYS[@]} -gt 0 ]]; then
  echo "found ${#ALEGREYA_KEYS[@]} variant(s): ${ALEGREYA_KEYS[*]}"
  for needed in AlegreyaR AlegreyaI AlegreyaB AlegreyaBI; do
    _v="ALEGREYA_${needed}"
    if [[ -z "${!_v}" ]]; then
      echo "warning: $needed not found (mom may fall back to a substitute)"
    fi
  done
  for groff_name in "${ALEGREYA_KEYS[@]}"; do
    case "$groff_name" in
      *I|*BI) opts="-i50" ;;
      *)      opts="-i0" ;;
    esac
    _v="ALEGREYA_${groff_name}"
    install_font_variant "$groff_name" "${!_v}" "$opts"
  done
  echo "Alegreya done. Installed: ${ALEGREYA_KEYS[*]}"
else
  echo "Alegreya: no OTF files found, skipping."
  echo "Install the package (e.g. sudo pacman -S otf-alegreya) and re-run." 1>&2
fi

# ── Grenze Gothisch ────────────────────────────────────────────────────────────
echo ""
echo "── Grenze Gothisch ───────────────────────────────────────────────────────────"
echo "searching for Grenze Gothisch OTF fonts..."
GRENZEGOTHISCH_KEYS=()  # tracks which groff-names have been found (bash 3.2 compat)

if [[ "$OS" == "Darwin" ]]; then
  GRENZEGOTHISCH_SEARCH_DIRS=(
    "$HOME/Library/Fonts"
    "/Library/Fonts"
  )
else
  GRENZEGOTHISCH_SEARCH_DIRS=(
    "$HOME/.fonts"
    "$HOME/.local/share/fonts"
    "/usr/share/fonts"
    "/usr/local/share/fonts"
  )
fi

_scan_grenzegothisch() {
  GRENZEGOTHISCH_KEYS=()
  for dir in "${GRENZEGOTHISCH_SEARCH_DIRS[@]}"; do
    [[ -d "$dir" ]] || continue
    local _before=${#GRENZEGOTHISCH_KEYS[@]}
    # Look for both .otf files and variable-weight .ttf files (e.g. GrenzeGotisch[wght].ttf)
    while IFS= read -r font; do
      [[ -z "$font" ]] && continue
      local base
      base=$(basename "$font")
      # Remove common font file extensions and variable-weight suffixes
      base=${base%.otf}
      base=${base%.ttf}
      base=${base%\[wght\]}
      case "$base" in
        GrenzeGothisch-Regular)    GRENZEGOTHISCH_GrenzeGothischR="$font";  GRENZEGOTHISCH_KEYS+=(GrenzeGothischR)  ;;
        GrenzeGothisch-Bold)       GRENZEGOTHISCH_GrenzeGothischB="$font";  GRENZEGOTHISCH_KEYS+=(GrenzeGothischB)  ;;
        GrenzeGothisch-Italic)     GRENZEGOTHISCH_GrenzeGothischI="$font";  GRENZEGOTHISCH_KEYS+=(GrenzeGothischI)  ;;
        GrenzeGothisch-BoldItalic) GRENZEGOTHISCH_GrenzeGothischBI="$font"; GRENZEGOTHISCH_KEYS+=(GrenzeGothischBI) ;;
        GrenzeGotisch)             GRENZEGOTHISCH_GrenzeGothischR="$font";  GRENZEGOTHISCH_KEYS+=(GrenzeGothischR)  ;;
      esac
    done < <(find "$dir" \( -name "GrenzeGothisch-*.otf" -o -name "GrenzeGotisch*.ttf" \) 2>/dev/null)
    [[ ${#GRENZEGOTHISCH_KEYS[@]} -gt $_before ]] && break
  done
}

_scan_grenzegothisch

if [[ ${#GRENZEGOTHISCH_KEYS[@]} -gt 0 ]]; then
  echo "found ${#GRENZEGOTHISCH_KEYS[@]} variant(s): ${GRENZEGOTHISCH_KEYS[*]}"
  for needed in GrenzeGothischR GrenzeGothischI GrenzeGothischB GrenzeGothischBI; do
    _v="GRENZEGOTHISCH_${needed}"
    if [[ -z "${!_v}" ]]; then
      echo "warning: $needed not found (mom may fall back to a substitute)"
    fi
  done
  for groff_name in "${GRENZEGOTHISCH_KEYS[@]}"; do
    case "$groff_name" in
      *I|*BI) opts="-i50" ;;
      *)      opts="-i0" ;;
    esac
    _v="GRENZEGOTHISCH_${groff_name}"
    install_font_variant "$groff_name" "${!_v}" "$opts"
  done
  echo "Grenze Gothisch done. Installed: ${GRENZEGOTHISCH_KEYS[*]}"
else
  echo "Grenze Gothisch: no font files found, skipping."
  echo "Install the package (e.g. sudo pacman -S otf-grenze-gothisch) and re-run." 1>&2
fi
