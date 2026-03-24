#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
seconds_per_target="${QS_FUZZ_SECONDS:-900}"
targets_raw="${QS_FUZZ_TARGETS:-decode encode decode_pairs}"
extra_args_raw="${QS_FUZZ_ARGS:-}"
cleanup_on_success="${QS_FUZZ_CLEANUP:-0}"

read -r -a targets <<< "$targets_raw"
read -r -a extra_args <<< "$extra_args_raw"

if ! [[ "$seconds_per_target" =~ ^[0-9]+$ ]] || [[ "$seconds_per_target" -le 0 ]]; then
  echo "error: QS_FUZZ_SECONDS must be a positive integer, got '$seconds_per_target'" >&2
  exit 1
fi

if ! cargo +nightly fuzz --help >/dev/null 2>&1; then
  cat >&2 <<'EOF'
error: cargo +nightly fuzz is required for fuzz soaks.

Install it with:
  rustup toolchain install nightly
  cargo install cargo-fuzz
EOF
  exit 1
fi

for target in "${targets[@]}"; do
  case "$target" in
    decode|encode|decode_pairs) ;;
    *)
      echo "error: unsupported fuzz target '$target'" >&2
      echo "supported targets: decode encode decode_pairs" >&2
      exit 1
      ;;
  esac
done

tmp_root="$(mktemp -d /tmp/qs_rust_fuzz_soak.XXXXXX)"
success=0

cleanup() {
  if [[ "$cleanup_on_success" == "1" && "$success" == "1" ]]; then
    rm -rf "$tmp_root"
  fi
}
trap cleanup EXIT

cd "$repo_root"

printf 'Fuzz soak root: %s\n' "$tmp_root"
printf 'Seconds per target: %s\n' "$seconds_per_target"
printf 'Targets: %s\n' "${targets[*]}"

if [[ ${#extra_args[@]} -gt 0 ]]; then
  printf 'Extra libFuzzer args: %s\n' "${extra_args[*]}"
fi

for target in "${targets[@]}"; do
  corpus_src="fuzz/corpus/$target"
  if [[ ! -d "$corpus_src" ]]; then
    echo "error: missing committed corpus '$corpus_src'" >&2
    exit 1
  fi

  run_root="$tmp_root/$target"
  corpus_work="$run_root/corpus"
  artifacts_work="$run_root/artifacts"

  mkdir -p "$corpus_work" "$artifacts_work"
  cp -R "$corpus_src"/. "$corpus_work"/

  cmd=(
    cargo +nightly fuzz run "$target" "$corpus_work" --
    "-artifact_prefix=${artifacts_work}/"
    "-max_total_time=${seconds_per_target}"
    "-verbosity=0"
    "-print_final_stats=1"
  )
  if [[ ${#extra_args[@]} -gt 0 ]]; then
    cmd+=("${extra_args[@]}")
  fi

  printf '\n== %s ==\n' "$target"
  printf 'Corpus copy: %s\n' "$corpus_work"
  printf 'Artifact dir: %s\n' "$artifacts_work"
  printf 'Command: '
  printf '%q ' "${cmd[@]}"
  printf '\n'

  "${cmd[@]}"
done

success=1
printf '\nFuzz soak completed successfully.\n'
if [[ "$cleanup_on_success" == "1" ]]; then
  printf 'Temporary corpora and artifacts were removed.\n'
else
  printf 'Temporary corpora and artifacts remain under %s\n' "$tmp_root"
  printf 'They are disposable and can be removed once the soak results are no longer needed.\n'
fi
