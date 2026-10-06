#!/usr/bin/env bash
# Make the NVIDIA GPU usable: the 595-open driver userspace is installed, but no kernel module exists
# for the running kernel, so `nvidia-smi` cannot talk to the driver. Run with: sudo bash eval/setup-gpu.sh
set -euo pipefail

if [ "$(id -u)" -ne 0 ]; then
  echo "run as root: sudo bash $0" >&2
  exit 1
fi

KVER="$(uname -r)"
DRV=595-open
echo "kernel: $KVER   driver: nvidia-$DRV"

if command -v mokutil >/dev/null && mokutil --sb-state 2>/dev/null | grep -qi "SecureBoot enabled"; then
  echo "WARNING: Secure Boot is enabled. Unsigned modules will not load; the DKMS path below will"
  echo "         ask you to enroll a MOK key (reboot + confirm). The prebuilt signed package avoids this."
fi

apt-get update -y

# 1) Prebuilt, signed modules for this exact kernel (preferred).
if apt-cache show "linux-modules-nvidia-$DRV-$KVER" >/dev/null 2>&1; then
  apt-get install -y "linux-modules-nvidia-$DRV-$KVER"
else
  # 2) Build with DKMS against the running kernel.
  echo "no prebuilt module package for $KVER; building with DKMS"
  apt-get install -y dkms "linux-headers-$KVER" "nvidia-dkms-$DRV"
fi

depmod -a
modprobe nvidia
modprobe nvidia_uvm || true

echo
nvidia-smi
echo
echo "GPU is up. CUDA toolkit: $(ls -d /usr/local/cuda* 2>/dev/null | tr '\n' ' ')"
echo "If nvidia-smi still fails: reboot, then re-run it (stale GPU state or a MOK enrollment is pending)."
