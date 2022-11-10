set -ex

curl -LsSf https://get.nexte.st/latest/linux | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin
cargo build -p jormungandr
cargo build -p jcli
cargo nextest run --no-fail-fast --partition hash:$1/$2 

