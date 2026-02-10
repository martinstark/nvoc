# NVOC - NVIDIA GPU Overclocking

GPU overclocking/undervolting utility for Blackwell RTX 50-series on Linux.

Born out of my frustration with the lack of an API that is both easy to use in the terminal, and easy to script around.

## Requirements

- Linux x86_64
- RTX 50-series GPU (5090, 5080, 5070, 5060)
- nvidia-open 550+ driver
- nvidia-utils package
- Root access

## Install

### AUR (Arch Linux)

```bash
paru -S nvoc-cli
```

### From source

```bash
# Install dependencies (Arch Linux)
sudo pacman -S nvidia-open nvidia-utils

# Build and install
cargo build --release
sudo cp target/release/nvoc /usr/local/bin/
```

## Usage

```bash
# Show GPU information
nvoc info

# OC
sudo nvoc -c MIN,MAX -o OFFSET -m MEM_OFFSET -p POWER_LIMIT

# Reset
sudo nvoc reset

# Dry Run
nvoc -c 200,2800 --dry-run
```

### Options

- `-c, --clocks <MIN,MAX>` - Set GPU locked clocks (MHz)
- `-o, --offset <OFFSET>` - Graphics clock offset (MHz)
- `-m, --memory-offset <OFFSET>` - Memory clock offset (MHz)
- `-p, --power <PERCENT>` - Power limit percentage (50-150%)
- `-d, --device <INDEX>` - GPU device index (default: 0)
- `--dry-run` - Preview changes only

### Examples

```bash
# 5090 uv example
sudo nvoc -c 200,2820 -o 856 -m 2000 -p 105

# Graphics offset
sudo nvoc -o 200

# Memory offset
sudo nvoc -m 1500

# Power limit
sudo nvoc -p 105

# Locked clocks
sudo nvoc -c 200,2800
```

Power limits are percentages of the GPU's default power limit. Hardware enforces absolute min/max constraints regardless of percentage.

### Info

```
$ nvoc info
driver: 590.48.01
gpu 0: NVIDIA GeForce RTX 5090
gpu clock: 1072MHz
gpu offset: 856MHz
mem clock: 405MHz
temp: 44°C
power: 14W
power limit: 600W (104%)
power range: 400W-575W (600W hard limit)
```

### Monitor

```bash
watch -n 1 nvoc info
```

## Limitations

The NVML API only supports global clock offsets, not per-voltage-point adjustments. Fine-grained undervolting (setting a specific frequency at a specific voltage) is not possible. Tools like MSI Afterburner achieve this through a non-public API. This is an NVML limitation, not specific to `nvoc`.
