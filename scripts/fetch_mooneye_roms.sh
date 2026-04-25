#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
submodule_dir="$repo_root/mooneye-test-suite"
out_dir="$repo_root/mooneye-roms"
tmp_out_dir="$repo_root/mooneye-roms.tmp"

if [ ! -d "$submodule_dir" ]; then
    echo "Missing mooneye-test-suite submodule. Run: git submodule update --init --recursive" >&2
    exit 1
fi

commit="$(git -C "$submodule_dir" rev-parse --short=7 HEAD)"

case "$commit" in
    443f6e1)
        package="mts-20240926-1737-443f6e1"
        ;;
    *)
        echo "No known prebuilt Mooneye package for submodule commit $commit." >&2
        echo "Update this script with the matching package from https://gekkio.fi/files/mooneye-test-suite/." >&2
        exit 1
        ;;
esac

url="https://gekkio.fi/files/mooneye-test-suite/$package/$package.tar.xz"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

archive="$tmp_dir/$package.tar.xz"
extract_dir="$tmp_dir/extract"

echo "Downloading $url"
curl -fL -o "$archive" "$url"

rm -rf "$tmp_out_dir"
mkdir -p "$extract_dir" "$tmp_out_dir"

tar -xJf "$archive" -C "$extract_dir"
cp -R "$extract_dir/$package/." "$tmp_out_dir/"

rm -rf "$out_dir"
mv "$tmp_out_dir" "$out_dir"

rom_count="$(find "$out_dir" -type f -name '*.gb' | wc -l | tr -d ' ')"
echo "Extracted $rom_count Mooneye ROMs into $out_dir"
