#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd -P)
repo_root=$(CDPATH= cd "$script_dir/.." && pwd -P)

version=$(awk '
    /^\[package\][[:space:]]*$/ { in_package = 1; next }
    /^\[/ { in_package = 0 }
    in_package && /^[[:space:]]*version[[:space:]]*=/ {
        if (match($0, /"[^"]+"/)) {
            print substr($0, RSTART + 1, RLENGTH - 2)
            exit
        }
    }
' "$repo_root/crates/arcadium-cli/Cargo.toml")
if [ -z "$version" ]; then
    echo 'Could not determine Arcadium CLI version from Cargo.toml.' >&2
    exit 1
fi

host=$(rustc -vV | sed -n 's/^host: //p')
case $host in
    x86_64-unknown-linux-gnu) platform=linux-x86_64 ;;
    aarch64-apple-darwin) platform=macos-aarch64 ;;
    x86_64-apple-darwin) platform=macos-x86_64 ;;
    *) echo "Unsupported release host: $host" >&2; exit 1 ;;
esac

for game in pong snake tictactoe; do
    if [ ! -f "$repo_root/bundled-games/$game.adm" ]; then
        echo "Missing bundled game: bundled-games/$game.adm" >&2
        exit 1
    fi
done

(cd "$repo_root" && cargo build --release -p arcadium-cli)
if [ ! -x "$repo_root/target/release/arcadium" ]; then
    echo 'Release build did not produce executable target/release/arcadium.' >&2
    exit 1
fi

release_dir=arcadium-v$version
staging_root=$(mktemp -d "${TMPDIR:-/tmp}/arcadium-release.XXXXXX")
archive_tmp=
trap 'rm -rf "$staging_root"; if [ -n "$archive_tmp" ]; then rm -f "$archive_tmp"; fi' 0
trap 'exit 1' 1 2 3 15

bundle=$staging_root/$release_dir
mkdir -p "$bundle/bundled-games"
cp "$repo_root/target/release/arcadium" "$bundle/arcadium"
cp "$repo_root/install.sh" "$bundle/install.sh"
cp "$repo_root/LICENSE" "$bundle/LICENSE"
for game in pong snake tictactoe; do
    cp "$repo_root/bundled-games/$game.adm" "$bundle/bundled-games/$game.adm"
done
chmod 755 "$bundle/arcadium" "$bundle/install.sh"

dist_dir=$repo_root/dist
mkdir -p "$dist_dir"
archive_name=$release_dir-$platform.tar.gz
archive_tmp=$(mktemp "$dist_dir/.$archive_name.XXXXXX")
tar -czf "$archive_tmp" -C "$staging_root" "$release_dir"
chmod 644 "$archive_tmp"
mv -f "$archive_tmp" "$dist_dir/$archive_name"
archive_tmp=

printf 'Built Arcadium release package:\n%s\n' "$dist_dir/$archive_name"
