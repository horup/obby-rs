if [ -d "dist" ]; then
    rm dist -r
fi
mkdir dist
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/obby.wasm dist/obby.wasm
cp index.html dist/index.html
cp mq_js_bundle.js dist/mq_js_bundle.js
cp favicon.png dist/favicon.png
cp -r res dist/res