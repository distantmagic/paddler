#!/bin/sh

# enter the script directory
cd "$(dirname "$0")"

./node_modules/.bin/esbuild \
    --bundle \
    --asset-names="./[name]" \
    --entry-names="./[name]" \
    --format=esm \
    --loader:.jpg=file \
    --loader:.otf=file \
    --loader:.svg=file \
    --loader:.ttf=file \
    --loader:.webp=file \
    --metafile=esbuild-meta-mgmt.json \
    --minify \
    --outdir=static \
    --sourcemap \
    --splitting \
    --target=safari16 \
    --tree-shaking=true \
    resources/css/mgmt-dashboard.css \
    resources/ts/global_stimulus.ts \
    resources/ts/global_turbo.ts \
    resources/ts/controller_refresh_body.ts \
;
