set -e

DIR=$(dirname $(realpath -s "$0"))
cp $DIR/mtproto.tl $DIR/cattlc/schema.tl
cd $DIR/cattlc
cargo run
cp $DIR/cattlc/schema.rs $DIR/cattl/src/mtproto.rs
