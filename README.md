# espflash-coredump

This is a convenience espflash log-processor for the `coredump*` feature in `esp-backtrace`. (see https://github.com/esp-rs/esp-hal/pull/2672)

You need `espflash` commit `4f8a526e23cf16223bd06e391807ae5fb7f18913` or later. (i.e. a version later than `3.2.0`)

## Example

```text
❯ espflash flash --monitor --processors=espflash-coredump examples\target\riscv32imac-unknown-none-elf\release\gpio_interrupt
[2024-12-04T12:25:19Z INFO ] Serial port: 'COM13'
[2024-12-04T12:25:19Z INFO ] Connecting...
[2024-12-04T12:25:19Z INFO ] Using flash stub
[2024-12-04T12:25:20Z WARN ] Setting baud rate higher than 115,200 can cause issues
Chip type:         esp32c6 (revision v0.0)
Crystal frequency: 40 MHz
Flash size:        4MB
Features:          WiFi 6, BT 5
MAC address:       60:55:f9:f6:01:78
App/part. size:    31,536/4,128,768 bytes, 0.76%
[2024-12-04T12:25:21Z INFO ] Segment at address '0x0' has not changed, skipping write
[2024-12-04T12:25:21Z INFO ] Segment at address '0x8000' has not changed, skipping write
[2024-12-04T12:25:21Z INFO ] Segment at address '0x10000' has not changed, skipping write
[2024-12-04T12:25:21Z INFO ] Flashing has completed!
Commands:
    CTRL+R    Reset chip
    CTRL+C    Exit

ESP-ROM:esp32c6-20220919
Build:Sep 19 2022
rst:0x1 (POWERON),boot:0xc (SPI_FAST_FLASH_BOOT)
SPIWP:0xee
mode:DIO, clock div:2
load:0x4086c410,len:0xd48
load:0x4086e610,len:0x2d68
load:0x40875720,len:0x1800
entry 0x4086c410
I (23) boot: ESP-IDF v5.1-beta1-378-gea5e0ff298-dirt 2nd stage bootloader
I (24) boot: compile time Jun  7 2023 08:02:08
I (25) boot: chip revision: v0.0
I (29) boot.esp32c6: SPI Speed      : 40MHz
I (33) boot.esp32c6: SPI Mode       : DIO
I (38) boot.esp32c6: SPI Flash Size : 4MB
I (43) boot: Enabling RNG early entropy source...
I (49) boot: Partition Table:
I (52) boot: ## Label            Usage          Type ST Offset   Length
I (59) boot:  0 nvs              WiFi data        01 02 00009000 00006000
I (67) boot:  1 phy_init         RF data          01 01 0000f000 00001000
I (74) boot:  2 factory          factory app      00 00 00010000 003f0000
I (82) boot: End of partition table
I (86) esp_image: segment 0: paddr=00010020 vaddr=42000020 size=058e8h ( 22760) map
I (99) esp_image: segment 1: paddr=00015910 vaddr=40800000 size=0057ch (  1404) load
I (103) esp_image: segment 2: paddr=00015e94 vaddr=42005e94 size=01124h (  4388) map
I (112) esp_image: segment 3: paddr=00016fc0 vaddr=4080057c size=00b48h (  2888) load
I (120) boot: Loaded app from partition at offset 0x10000
I (125) boot: Disabling RNG early entropy source...


====================== PANIC ======================
panicked at src\bin\gpio_interrupt.rs:55:5:
Panic message here!





Receiving coredump ...
Got coredump
Run `riscv32-esp-elf-gdb examples\target\riscv32imac-unknown-none-elf\release\gpio_interrupt coredump.elf`
```
