#!/usr/bin/env bash
# bench_compare.sh — Compare gapline validate against MobilityData/gtfs-validator
# (the canonical GTFS validator, written in Java) across multiple feed sizes.
#
# Iterates over 4 tiers: small / medium / large / huge. For each tier, runs
# 4 cases: {gapline, gtfs-validator} × {zip, directory}. Both tools accept
# either input form and the cost profiles differ (zip decompression vs fs walk).
#
# Metrics:
#   - Wall-clock time: hyperfine (warmup + N runs, mean±stddev)
#   - Peak RSS (max resident set size): single-run each
#       Linux → /proc/$pid/status VmHWM (kernel-tracked, monotone, exact)
#       other → ps -o rss sampling at 50ms intervals (best-effort)
#
# Usage:
#   scripts/bench_compare.sh
#   FEED_TIERS=small,medium scripts/bench_compare.sh
#   VALIDATOR_VERSION=5.0.1 scripts/bench_compare.sh
#
# Env overrides:
#   FEED_TIERS          comma-separated subset: small,medium,large,huge
#                       (default: all 4)
#   VALIDATOR_VERSION   MobilityData tag      (default: latest from GitHub API)
#   GAPLINE_BIN         path to gapline       (default: PATH → target/release)
#   JAVA_FLAGS          extra JVM flags       (default: empty — stock heap)
#   WARMUP, RUNS        hyperfine params      (default: 1, 5)
#
# Dependencies: java, hyperfine, curl, unzip, awk, (ps on non-Linux)
# Optional:     jq (for the final summary tables)

set -euo pipefail

# --- Paths ---
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DATA_DIR="$SCRIPT_DIR/bench-data"
RESULTS_DIR="$SCRIPT_DIR/bench-results"

# --- Config ---
JAVA_FLAGS="${JAVA_FLAGS:-}"
WARMUP="${WARMUP:-1}"
RUNS="${RUNS:-5}"
FEED_TIERS_FILTER="${FEED_TIERS:-small,medium,large,huge}"

# --- Feed tier catalog: "tier|label|url" ---
FEED_CATALOG=(
    "small|BART (SF Bay Area, ~2 MB)|https://www.bart.gov/dev/schedules/google_transit.zip"
    "medium|STM (Montréal, ~15 MB)|https://www.stm.info/sites/default/files/gtfs/gtfs_stm.zip"
    "large|MBTA (Boston, ~40 MB)|https://cdn.mbta.com/MBTA_GTFS.zip"
    "huge|OV Nederland (national NL, ~120 MB)|https://gtfs.ovapi.nl/nl/gtfs-nl.zip"
)

# --- Color helpers ---
if [ -t 1 ]; then
    GREEN='\033[0;32m'; YELLOW='\033[0;33m'; BLUE='\033[0;34m'
    RED='\033[0;31m'; BOLD='\033[1m'; RESET='\033[0m'
else
    GREEN=''; YELLOW=''; BLUE=''; RED=''; BOLD=''; RESET=''
fi
info() { printf "${BLUE}info${RESET} %s\n" "$1"; }
warn() { printf "${YELLOW}warn${RESET} %s\n" "$1"; }
err()  { printf "${RED}error${RESET} %s\n" "$1" >&2; }
ok()   { printf "${GREEN}ok${RESET}   %s\n" "$1"; }

# --- Dependency checks ---
need() { command -v "$1" >/dev/null 2>&1 || { err "missing dependency: $1"; exit 127; }; }
need java
need hyperfine
need curl
need unzip

# Peak RSS sampling backend
if [ "$(uname -s)" = "Linux" ] && [ -r /proc/self/status ]; then
    RSS_MODE="proc"
elif command -v ps >/dev/null 2>&1; then
    RSS_MODE="ps"
else
    err "cannot measure RSS: no /proc and no ps"
    exit 127
fi

# --- Resolve gapline binary ---
# Priority: explicit $GAPLINE_BIN → gapline on PATH (installed) → target/release
if [ -n "${GAPLINE_BIN:-}" ]; then
    GAPLINE_SOURCE="env"
elif GAPLINE_BIN="$(command -v gapline 2>/dev/null)" && [ -n "$GAPLINE_BIN" ]; then
    GAPLINE_SOURCE="PATH"
elif [ -x "$ROOT/target/release/headway" ]; then
    GAPLINE_BIN="$ROOT/target/release/headway"
    GAPLINE_SOURCE="target/release"
else
    err "no gapline binary found — install (scripts/install.sh) or build (cargo build --release)"
    exit 127
fi

if [ ! -x "$GAPLINE_BIN" ]; then
    err "gapline binary not executable: $GAPLINE_BIN"
    exit 127
fi

mkdir -p "$DATA_DIR" "$RESULTS_DIR"

# --- Resolve gtfs-validator release metadata ---
# Don't guess asset names — read the release JSON and pick the CLI jar from
# its assets. Naming has changed across versions (e.g. v7.x renamed the artifact).
if [ -n "${VALIDATOR_VERSION:-}" ]; then
    RELEASE_PATH="tags/v${VALIDATOR_VERSION}"
    info "fetching gtfs-validator release v${VALIDATOR_VERSION}"
else
    RELEASE_PATH="latest"
    info "fetching latest gtfs-validator release"
fi

RELEASE_JSON="$(curl -fsSL "https://api.github.com/repos/MobilityData/gtfs-validator/releases/${RELEASE_PATH}")" \
    || { err "failed to fetch release metadata"; exit 1; }

if [ -z "${VALIDATOR_VERSION:-}" ]; then
    VALIDATOR_VERSION="$(printf '%s' "$RELEASE_JSON" \
        | grep -oE '"tag_name":[[:space:]]*"v[^"]+"' \
        | head -n1 \
        | sed -E 's/.*"v([^"]+)".*/\1/')"
    [ -n "$VALIDATOR_VERSION" ] || { err "empty validator version"; exit 1; }
fi

# Pick the first jar asset whose name looks like the CLI build.
# Prefer names containing 'cli'; fall back to any .jar.
VALIDATOR_URL="$(printf '%s' "$RELEASE_JSON" \
    | grep -oE '"browser_download_url":[[:space:]]*"[^"]+\.jar"' \
    | sed -E 's/.*"(https[^"]+)".*/\1/' \
    | { grep -iE '(^|/)[^/]*cli[^/]*\.jar$' || cat; } \
    | head -n1)"

[ -n "$VALIDATOR_URL" ] || { err "no jar asset found in release v${VALIDATOR_VERSION}"; exit 1; }

VALIDATOR_JAR="$DATA_DIR/$(basename "$VALIDATOR_URL")"

if [ ! -f "$VALIDATOR_JAR" ]; then
    info "downloading gtfs-validator v${VALIDATOR_VERSION}"
    curl -fsSL -L --retry 3 -o "$VALIDATOR_JAR" "$VALIDATOR_URL"
    ok "saved $VALIDATOR_JAR"
fi

GAPLINE_VER="$("$GAPLINE_BIN" --version 2>/dev/null || echo "gapline ?")"

printf "\n${BOLD}== Setup ==${RESET}\n"
printf "  %-20s %s\n" "Gapline:"        "$GAPLINE_VER  [$GAPLINE_SOURCE: $GAPLINE_BIN]"
printf "  %-20s %s\n" "gtfs-validator:" "v$VALIDATOR_VERSION  ($VALIDATOR_JAR)"
printf "  %-20s %s\n" "Hyperfine:"      "warmup=$WARMUP runs=$RUNS"
printf "  %-20s %s\n" "RSS backend:"    "$RSS_MODE"
printf "  %-20s %s\n" "Tiers:"          "$FEED_TIERS_FILTER"
printf "\n"

# --- Peak RSS sampler ---
# On Linux, VmHWM is the kernel-maintained high-water mark of the resident
# set — monotone, so the last successful read before the process exits is
# the exact peak. Polling at 50ms is for timing only, not accuracy.
#
# On other platforms, ps reports instantaneous RSS, so peaks between samples
# are missed. Good enough for order-of-magnitude comparison.
measure_rss() {
    local name="$1"; shift
    local peak=0 v t0 t1 elapsed pid

    t0="${EPOCHREALTIME:-$(date +%s.%N)}"
    "$@" >/dev/null 2>&1 &
    pid=$!

    case "$RSS_MODE" in
        proc)
            while kill -0 "$pid" 2>/dev/null; do
                v="$(awk '/^VmHWM:/ {print $2; exit}' "/proc/$pid/status" 2>/dev/null || true)"
                if [ -n "$v" ] && [ "$v" -gt "$peak" ]; then peak="$v"; fi
                sleep 0.05
            done
            ;;
        ps)
            while kill -0 "$pid" 2>/dev/null; do
                v="$(ps -o rss= -p "$pid" 2>/dev/null | tr -d ' ' || true)"
                if [ -n "$v" ] && [ "$v" -gt "$peak" ]; then peak="$v"; fi
                sleep 0.05
            done
            ;;
    esac

    wait "$pid" 2>/dev/null || true
    t1="${EPOCHREALTIME:-$(date +%s.%N)}"
    elapsed="$(awk -v a="$t0" -v b="$t1" 'BEGIN { printf "%.3f", b - a }')"
    printf "  %-26s peak_rss=%10s KiB   wall=%ss\n" "$name" "${peak:-?}" "$elapsed"
}

# --- Per-feed benchmark ---
bench_feed() {
    local tier="$1" label="$2" url="$3"
    local feed_zip="$DATA_DIR/${tier}.zip"
    local feed_dir="$DATA_DIR/${tier}-unzipped"

    printf "\n${BOLD}== Tier: %s — %s ==${RESET}\n" "$tier" "$label"

    # Download + extract (cached)
    if [ ! -f "$feed_zip" ]; then
        info "downloading feed ($tier)"
        if ! curl -fsSL --retry 3 -o "$feed_zip" "$url"; then
            warn "download failed for $tier — skipping"
            rm -f "$feed_zip"
            return 1
        fi
        ok "saved $feed_zip"
    fi

    if [ ! -d "$feed_dir" ] || [ -z "$(ls -A "$feed_dir" 2>/dev/null)" ]; then
        info "extracting to $feed_dir"
        mkdir -p "$feed_dir"
        unzip -oq "$feed_zip" -d "$feed_dir"
    fi

    local zip_sz dir_sz
    zip_sz="$(du -h "$feed_zip" | cut -f1)"
    dir_sz="$(du -sh "$feed_dir" 2>/dev/null | cut -f1)"
    printf "  %-14s zip=%s  dir=%s\n\n" "sizes:" "$zip_sz" "$dir_sz"

    local hw_out_zip="$RESULTS_DIR/gapline-${tier}-zip.json"
    local hw_out_dir="$RESULTS_DIR/gapline-${tier}-dir.json"
    local gv_out_zip="$RESULTS_DIR/gtfs-validator-${tier}-zip-out"
    local gv_out_dir="$RESULTS_DIR/gtfs-validator-${tier}-dir-out"
    local hf_json_zip="$RESULTS_DIR/hyperfine-${tier}-zip.json"
    local hf_json_dir="$RESULTS_DIR/hyperfine-${tier}-dir.json"
    local rss_log="$RESULTS_DIR/rss-${tier}.txt"

    # Suffix each hyperfine cmd with `|| true` so the shell always exits 0:
    # validators return non-zero when the real-world feed has findings, which
    # is expected — we don't want hyperfine to emit "Ignoring non-zero exit"
    # warnings on every run.
    local hw_cmd_zip hw_cmd_dir gv_cmd_zip gv_cmd_dir clean_cmd
    hw_cmd_zip="$(printf '%q ' "$GAPLINE_BIN" validate -f "$feed_zip" --format json -o "$hw_out_zip")|| true"
    hw_cmd_dir="$(printf '%q ' "$GAPLINE_BIN" validate -f "$feed_dir" --format json -o "$hw_out_dir")|| true"
    # shellcheck disable=SC2086
    gv_cmd_zip="$(printf '%q ' java)${JAVA_FLAGS:+$JAVA_FLAGS }$(printf '%q ' -jar "$VALIDATOR_JAR" --input "$feed_zip" --output_base "$gv_out_zip")|| true"
    # shellcheck disable=SC2086
    gv_cmd_dir="$(printf '%q ' java)${JAVA_FLAGS:+$JAVA_FLAGS }$(printf '%q ' -jar "$VALIDATOR_JAR" --input "$feed_dir" --output_base "$gv_out_dir")|| true"

    clean_cmd="rm -rf \
 $(printf '%q' "$hw_out_zip") \
 $(printf '%q' "$hw_out_dir") \
 $(printf '%q' "$gv_out_zip") \
 $(printf '%q' "$gv_out_dir")"

    info "wall-clock — zip input"
    eval "$clean_cmd"
    hyperfine \
        --warmup "$WARMUP" --runs "$RUNS" \
        --export-json "$hf_json_zip" \
        --prepare "$clean_cmd" \
        -n "gapline (zip)"        "$hw_cmd_zip" \
        -n "gtfs-validator (zip)" "$gv_cmd_zip"

    printf "\n"
    info "wall-clock — dir input"
    eval "$clean_cmd"
    hyperfine \
        --warmup "$WARMUP" --runs "$RUNS" \
        --export-json "$hf_json_dir" \
        --prepare "$clean_cmd" \
        -n "gapline (dir)"        "$hw_cmd_dir" \
        -n "gtfs-validator (dir)" "$gv_cmd_dir"

    printf "\n"
    info "peak RSS via ${RSS_MODE} sampling (1 run each)"
    : > "$rss_log"

    eval "$clean_cmd"
    measure_rss "gapline (zip)"        "$GAPLINE_BIN" validate -f "$feed_zip" --format json -o "$hw_out_zip" | tee -a "$rss_log"
    eval "$clean_cmd"
    measure_rss "gapline (dir)"        "$GAPLINE_BIN" validate -f "$feed_dir" --format json -o "$hw_out_dir" | tee -a "$rss_log"
    eval "$clean_cmd"
    # shellcheck disable=SC2086
    measure_rss "gtfs-validator (zip)" java $JAVA_FLAGS -jar "$VALIDATOR_JAR" --input "$feed_zip" --output_base "$gv_out_zip" | tee -a "$rss_log"
    eval "$clean_cmd"
    # shellcheck disable=SC2086
    measure_rss "gtfs-validator (dir)" java $JAVA_FLAGS -jar "$VALIDATOR_JAR" --input "$feed_dir" --output_base "$gv_out_dir" | tee -a "$rss_log"
}

# --- Summary extraction helpers ---
# Mean time (seconds) for a named command from a hyperfine JSON file.
hf_field() {
    local json="$1" name="$2" field="$3"
    [ -f "$json" ] || { echo ""; return; }
    jq -r --arg n "$name" --arg f "$field" \
        '(.results[] | select(.command == $n) | .[$f]) // empty' "$json" 2>/dev/null
}

# Peak RSS (KiB) for a named case from the RSS log.
rss_kib() {
    local log="$1" name="$2"
    [ -f "$log" ] || { echo ""; return; }
    awk -v n="$name" '
        index($0, n) > 0 && match($0, /peak_rss=[0-9]+/) {
            print substr($0, RSTART+9, RLENGTH-9)
            exit
        }
    ' "$log"
}

fmt_time() {
    local mean="$1" stddev="$2"
    if [ -z "$mean" ]; then echo "—"; return; fi
    awk -v m="$mean" -v s="${stddev:-0}" \
        'BEGIN { printf "%.1fms ± %.1fms", m*1000, s*1000 }'
}

fmt_rss() {
    local kib="$1"
    if [ -z "$kib" ] || [ "$kib" = "0" ]; then echo "—"; return; fi
    awk -v k="$kib" 'BEGIN {
        if      (k >= 1048576) printf "%.2f GiB", k/1048576
        else if (k >= 1024)    printf "%.1f MiB", k/1024
        else                   printf "%d KiB",   k
    }'
}

# Speed ratio: gv_mean / hw_mean → "Nx" (gapline is N times faster when > 1).
fmt_ratio() {
    local hw="$1" gv="$2"
    if [ -z "$hw" ] || [ -z "$gv" ]; then echo "—"; return; fi
    awk -v a="$gv" -v b="$hw" 'BEGIN {
        if (b+0 == 0) print "—"; else printf "%.1fx", a/b
    }'
}

print_summary_tables() {
    if ! command -v jq >/dev/null 2>&1; then
        warn "jq not found — skipping summary tables (install jq to enable)"
        return 0
    fi

    printf "\n${BOLD}== Wall-clock (mean ± stddev) ==${RESET}\n"
    printf "%-10s  %-20s  %-22s  %-7s  %-20s  %-22s  %-7s\n" \
        "tier" "gapline (zip)" "gtfs-validator (zip)" "speedup" \
        "gapline (dir)" "gtfs-validator (dir)" "speedup"

    local tier zj dj hw_zm gv_zm hw_zs gv_zs hw_dm gv_dm hw_ds gv_ds
    for tier in "${RAN_TIERS[@]}"; do
        zj="$RESULTS_DIR/hyperfine-${tier}-zip.json"
        dj="$RESULTS_DIR/hyperfine-${tier}-dir.json"
        hw_zm="$(hf_field "$zj" "gapline (zip)"        mean)"
        hw_zs="$(hf_field "$zj" "gapline (zip)"        stddev)"
        gv_zm="$(hf_field "$zj" "gtfs-validator (zip)" mean)"
        gv_zs="$(hf_field "$zj" "gtfs-validator (zip)" stddev)"
        hw_dm="$(hf_field "$dj" "gapline (dir)"        mean)"
        hw_ds="$(hf_field "$dj" "gapline (dir)"        stddev)"
        gv_dm="$(hf_field "$dj" "gtfs-validator (dir)" mean)"
        gv_ds="$(hf_field "$dj" "gtfs-validator (dir)" stddev)"

        printf "%-10s  %-20s  %-22s  %-7s  %-20s  %-22s  %-7s\n" \
            "$tier" \
            "$(fmt_time "$hw_zm" "$hw_zs")" \
            "$(fmt_time "$gv_zm" "$gv_zs")" \
            "$(fmt_ratio "$hw_zm" "$gv_zm")" \
            "$(fmt_time "$hw_dm" "$hw_ds")" \
            "$(fmt_time "$gv_dm" "$gv_ds")" \
            "$(fmt_ratio "$hw_dm" "$gv_dm")"
    done

    printf "\n${BOLD}== Peak RSS ==${RESET}\n"
    printf "%-10s  %-15s  %-20s  %-15s  %-20s\n" \
        "tier" "gapline (zip)" "gtfs-validator (zip)" "gapline (dir)" "gtfs-validator (dir)"

    local log
    for tier in "${RAN_TIERS[@]}"; do
        log="$RESULTS_DIR/rss-${tier}.txt"
        printf "%-10s  %-15s  %-20s  %-15s  %-20s\n" \
            "$tier" \
            "$(fmt_rss "$(rss_kib "$log" "gapline (zip)")")" \
            "$(fmt_rss "$(rss_kib "$log" "gtfs-validator (zip)")")" \
            "$(fmt_rss "$(rss_kib "$log" "gapline (dir)")")" \
            "$(fmt_rss "$(rss_kib "$log" "gtfs-validator (dir)")")"
    done
}

# --- Main loop ---
tier_selected() {
    case ",${FEED_TIERS_FILTER}," in
        *",$1,"*) return 0 ;;
        *)        return 1 ;;
    esac
}

declare -a RAN_TIERS=()
for entry in "${FEED_CATALOG[@]}"; do
    tier="${entry%%|*}"
    rest="${entry#*|}"
    label="${rest%%|*}"
    url="${rest#*|}"
    tier_selected "$tier" || continue
    if bench_feed "$tier" "$label" "$url"; then
        RAN_TIERS+=("$tier")
    fi
done

if [ "${#RAN_TIERS[@]}" -eq 0 ]; then
    err "no tiers ran successfully"
    exit 1
fi

print_summary_tables

printf "\n${BOLD}== Files ==${RESET}\n"
ok "completed tiers: ${RAN_TIERS[*]}"
printf "  %-18s %s\n" "results dir:" "$RESULTS_DIR"
printf "  %-18s %s\n" "hyperfine JSON:" "$RESULTS_DIR/hyperfine-<tier>-{zip,dir}.json"
printf "  %-18s %s\n" "peak RSS logs:"  "$RESULTS_DIR/rss-<tier>.txt"
