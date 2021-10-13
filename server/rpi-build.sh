#!/bin/sh
cross build --target armv7-unknown-linux-gnueabihf --release

rsync -aczvhP target/armv7-unknown-linux-gnueabihf/release/pie-server pi@$PI:~
