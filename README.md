# NVOC - NVIDIA GPU Overclocking

GPU overclocking/undervolting utility for Blackwell RTX 50-series on Linux.

Born out of my frustration with the lack of an API that is both easy to use in the terminal, and easy to script around.

**WARNING: Overclocking may damage hardware.**

## Requirements

- Linux x86_64
- RTX 50-series GPU (5090, 5080, 5070, 5060)
- nvidia-open 550+ driver
- nvidia-utils package
- Root access

## Install

```bash
# Install dependencies (Arch Linux)
sudo pacman -S nvidia-open nvidia-utils

# Build and install
cargo build --release
sudo cp target/release/nvoc /usr/local/bin/
```

## Usage

### Commands

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

### Commands

- `info`
- `reset`

## Limitations



## Examples

### Overclocking or Undervolting
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

## Power Management

Power limits are specified as percentages of the GPU's default power limit:

- **100%**: default hardware power limit
- **110%**: 10% increase over default

Hardware enforces absolute min/max constraints regardless of percentage.

## Info Command Output

```bash
$ nvoc info
Driver 580.82.09
0: NVIDIA GeForce RTX 5090
Blackwell v16777240
GPU: 1177MHz
GPU Offset: 960MHz
Mem: 15501MHz
Temp: 56°C
Power: 45W
Power Limit: 600W (104% of default)
Power Range: 400W-575W (hard limit: 600W)
```

## Limitations

### Voltage Curve Offsets

The NVML API only supports global clock offsets, not per-voltage point adjustments. This means:

- Offsets apply uniformly across all voltage points on the GPU's voltage/frequency curve
- Fine-grained undervolting (setting specific voltage for specific frequency) is not possible
- Tools like MSI Afterburner's curve editor provide more granular control via a non-public API

This is a hardware abstraction layer limitation, not specific to `nvoc`.

## Monitor

```bash
watch -n 1 nvoc info
```
