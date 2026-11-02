#!/bin/sh
# Starts the backend without root rights. Older installations created the uploads volume as root,
# so its ownership is fixed once before the privileges are dropped.
set -eu
if [ "$(id -u)" = "0" ]; then
    mkdir -p /app/uploads
    if [ "$(stat -c %u /app/uploads)" != "10001" ]; then
        chown -R shop:shop /app/uploads
    fi
    exec setpriv --reuid=shop --regid=shop --init-groups /app/backend "$@"
fi
exec /app/backend "$@"
