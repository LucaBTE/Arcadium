#!/usr/bin/env sh
set -eu

case $(uname -s) in
    Linux) platform=linux ;;
    Darwin) platform=macos ;;
    *) echo 'Arcadium supports Linux and macOS only.' >&2; exit 1 ;;
esac

case ${HOME-} in
    /*) ;;
    *) echo 'HOME must be an absolute path.' >&2; exit 1 ;;
esac

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd -P)
if [ ! -f "$script_dir/arcadium" ] || [ ! -d "$script_dir/bundled-games" ]; then
    echo 'Release bundle must contain arcadium and bundled-games/ beside install.sh.' >&2
    exit 1
fi

set -- "$script_dir"/bundled-games/*.adm
if [ ! -f "$1" ]; then
    echo 'Release bundle has no bundled .adm games.' >&2
    exit 1
fi

if [ "$platform" = macos ]; then
    data_dir=$HOME/Library/Application\ Support/Arcadium
elif [ "${XDG_DATA_HOME+x}" = x ]; then
    case $XDG_DATA_HOME in
        /*) data_dir=$XDG_DATA_HOME/arcadium ;;
        *) echo 'XDG_DATA_HOME must be an absolute path.' >&2; exit 1 ;;
    esac
else
    data_dir=$HOME/.local/share/arcadium
fi

bin_dir=$HOME/.local/bin
mkdir -p "$bin_dir" "$data_dir/bundled-games" "$data_dir/games"
cp "$script_dir/arcadium" "$bin_dir/arcadium"
chmod +x "$bin_dir/arcadium"
cp "$@" "$data_dir/bundled-games/"

echo "Installed Arcadium to $bin_dir/arcadium"
echo "Bundled games: $data_dir/bundled-games"
case :$PATH: in
    *:"$bin_dir":*) ;;
    *)
        echo 'Add ~/.local/bin to your PATH, then restart your terminal:'
        echo '  export PATH="$HOME/.local/bin:$PATH"'
        ;;
esac
