cargo build --release && \
rust-objcopy -O binary target/thumbv6m-none-eabi/release/spiky-rs target/spiky-rs.bin && \
~/.arduino15/packages/arduino/tools/bossac/1.7.0-arduino3/bossac -i -d -U true -i -e -w -v target/spiky-rs.bin -R 