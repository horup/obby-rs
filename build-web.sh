if [ -d "dist" ]; then
    rm dist -r
fi
mkdir dist
cargo build --target wasm32-unknown-unknown --release
mv target/wasm32-unknown-unknown/release/obby.wasm dist/obby.wasm
mv index.html dist/index.html
mv res dist/res