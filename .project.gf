[gdb]
path=./rust-gdb

[commands]
Compile ripe=shell cargo b --bin ripe --profile debugging
Run ripe=file target/debugging/ripe;run&