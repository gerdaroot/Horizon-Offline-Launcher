#!/usr/bin/env bash
# Installs Ubuntu packages on a GitHub runner: apt_install.sh PACKAGE...
#
# The runners' apt has failed for hours at a time (exit 100, or a 30 minute hang): a third-party
# source in the image (Microsoft's, a PPA) that doesn't answer fails the whole `apt-get update`,
# and so can the Azure mirror. So: short timeouts, and on each retry fewer places to fail.
set -u

opts=(-o Acquire::Retries=2 -o Acquire::http::Timeout=30 -o Acquire::https::Timeout=30)

try() {
    sudo apt-get "${opts[@]}" update && sudo apt-get "${opts[@]}" install -y --no-install-recommends "$@"
}

try "$@" && exit 0

echo "::warning::apt failed; retrying with Ubuntu's own sources only"
for f in /etc/apt/sources.list.d/*; do
    case "$(basename "$f")" in ubuntu.sources) ;; *) sudo rm -f "$f" ;; esac
done
sleep 10
try "$@" && exit 0

echo "::warning::apt failed again; retrying on archive.ubuntu.com instead of Azure's mirror"
sudo sed -i 's#http://azure\.archive\.ubuntu\.com#http://archive.ubuntu.com#g' \
    /etc/apt/sources.list /etc/apt/sources.list.d/ubuntu.sources 2>/dev/null
sleep 20
try "$@" && exit 0

echo "::error::apt-get could not install $*: Ubuntu's mirrors or GitHub's runners are failing; rerun later"
exit 1
