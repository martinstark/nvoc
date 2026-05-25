# NVOC - NVIDIA GPU Overclocking

GPU overclocking and undervolting utility for NVIDIA Blackwell GPUs on Linux (Help wanted with Ada Lovelace and Ampere GPU support, see [Issues](https://github.com/martinstark/nvoc/issues)).

Supports single and multi GPU setups. Filter GPUs by string matching, regex, or uuid.

Born out of my frustration with the lack of an API that is both easy to use in the terminal, and easy to script around.

## Requirements

- Linux
- Blackwell GPU (GeForce RTX 50-series or RTX PRO Blackwell)
- nvidia-open 555+ driver
- nvidia-utils package
- root access

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

# List GPUs
nvoc list

# OC
sudo nvoc -c MIN,MAX -o OFFSET -m MEM_OFFSET -p POWER_LIMIT

# Reset
sudo nvoc reset

# Dry Run
nvoc -c 200,2800 --dry-run
```

### Options

```py
-c, --clocks <MIN,MAX>       # GPU locked clocks (MHz)
-o, --offset <OFFSET>        # Graphics clock offset (MHz)
-m, --memory-offset <OFFSET> # Memory clock offset (MHz)
-p, --power <PERCENT>        # Power limit percentage of default
# use device flag on multi gpu systems, defaults to device 0
-d, --device <SELECTOR>      # GPU device selector:
                             # index | uuid | name:pattern | regex:pattern | all
--json                       # Emit JSON output (info, list)
--uuid                       # Emit device UUIDs separated by line break (list)
--dry-run                    # Preview changes
```

### Examples

```bash
# SINGLE GPU

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

```bash
# MULTI GPU

# Apply to all GPUs
sudo nvoc -d all -o 200

# Using Device ID (unstable across reboots)
sudo nvoc -d 0,1 -c 200,2820 -o 856 -m 2000 -p 105

# Using UUID (stable across reboots)
sudo nvoc -d GPU-1234... -o 856

# Using name pattern (case-insensitive substring match on device name)
sudo nvoc -d name:5090 -o 856
sudo nvoc -d "n:5060 ti" -o 100

# Using a regex pattern (case-sensitive regex search on device name)
sudo nvoc -d "regex:RTX 50[89]0" -o 856
# Optional ' Ti' suffix - matches both "RTX 5060" and "RTX 5060 Ti"
sudo nvoc -d "r:5060( Ti)?" -o 100
```

Power limits are percentages of the GPU's default power limit. `nvoc` does not enforce a fixed percentage range; the card's NVML min/max power limits decide the effective watts.

### Info

```
$ nvoc info
driver: 595.71.05
gpu 0: NVIDIA GeForce RTX 5090
gpu clock: 1080MHz
gpu offset: 856MHz
mem clock: 810MHz
mem offset: 2000MHz
temp: 52°C
power: 28W
power limit: 600W (104%)
power range: 400W-575W (600W hard limit)
```

```bash
# JSON
nvoc info --json
```

### List

```
$ nvoc list
0 - NVIDIA GeForce RTX 5090 - GPU-1234...
```

```bash
# UUIDs only, separated by line break, no labels
nvoc list --uuid

# JSON
nvoc list --json
```

### Monitor

```bash
watch -n 1 nvoc info
```

### Apply on Boot (systemd)

To apply settings on every boot, install a oneshot service:

```ini
# /etc/systemd/system/gpu-oc.service
[Unit]
Description=GPU overclock settings
After=multi-user.target

[Service]
Type=oneshot
ExecStart=/usr/bin/nvoc -c 200,2820 -o 856 -m 2000 -p 105

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now gpu-oc.service
```

Replace the `ExecStart` arguments with your tuned values. Adjust the binary path to `/usr/local/bin/nvoc` if you installed from source.

On multi-GPU systems, pin by UUID (`-d GPU-...`) or by model (`-d name:5090`, or `-d "r:50[89]0"` for a regex match). NVML device indices aren't guaranteed stable across reboots.

## Limitations

The NVML API only supports global clock offsets, not per-voltage-point adjustments. Fine-grained undervolting (setting a specific frequency at a specific voltage) is not possible. Tools like MSI Afterburner achieve this through a non-public API. This is an NVML limitation, not specific to `nvoc`.
