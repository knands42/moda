#!/usr/bin/env bash
set -euo pipefail

# Simple Windows VM creator for libvirt/virt-install.
# Default network: NAT (libvirt 'default').
# Optional network: an existing Linux bridge, e.g. BRIDGE_IF=br0.
#
# Why NAT by default?
# Bridging directly over a Wi-Fi client interface (like wlan0 in managed mode)
# usually does not work on Linux because the AP only expects one station MAC.
# If you only have wlan0, use NAT unless you already created a working bridge/macvtap setup.
#
# Optional GPU passthrough:
#   GPU_VIDEO_PCI=0000:01:00.0 GPU_AUDIO_PCI=0000:01:00.1 ./create_windows_vm.sh
#
# Example:
#   WIN_ISO=~/ISOs/Win11_24H2_English_x64.iso \
#   VIRTIO_ISO=~/ISOs/virtio-win.iso \
#   VM_NAME=win11-gaming \
#   RAM_MB=16384 VCPUS=12 DISK_GB=200 \
#   ./scripts/VMs/create_windows_vm.sh

VM_BASE_DIR="${VM_BASE_DIR:-$HOME/vm}"
VM_NAME="${VM_NAME:-win11}"
RAM_MB="${RAM_MB:-16384}"
VCPUS="${VCPUS:-12}"
DISK_GB="${DISK_GB:-160}"
CPU_MODE="${CPU_MODE:-host-passthrough}"
OS_VARIANT="${OS_VARIANT:-win11}"
WIN_ISO="${WIN_ISO:-$VM_BASE_DIR/Win11.iso}"
VIRTIO_ISO="${VIRTIO_ISO:-$VM_BASE_DIR/virtio-win.iso}"
POOL_DIR="${POOL_DIR:-$VM_BASE_DIR}"
DISK_PATH="${DISK_PATH:-$POOL_DIR/${VM_NAME}.qcow2}"
NETWORK_MODE="${NETWORK_MODE:-nat}"
BRIDGE_IF="${BRIDGE_IF:-}"
GPU_VIDEO_PCI="${GPU_VIDEO_PCI:-}"
GPU_AUDIO_PCI="${GPU_AUDIO_PCI:-}"
EXTRA_DISK_BUS="${EXTRA_DISK_BUS:-virtio}"
LIBVIRT_URI="${LIBVIRT_URI:-qemu:///system}"

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "Missing required command: $1" >&2
    exit 1
  }
}

fail() {
  echo "ERROR: $*" >&2
  exit 1
}

info() {
  echo "==> $*"
}

require_cmd virt-install
require_cmd virsh
require_cmd qemu-img
require_cmd systemctl

[[ -f "$WIN_ISO" ]] || fail "Windows ISO not found: $WIN_ISO"
[[ -f "$VIRTIO_ISO" ]] || fail "VirtIO ISO not found: $VIRTIO_ISO"

if [[ "$NETWORK_MODE" == "bridge" ]]; then
  [[ -n "$BRIDGE_IF" ]] || fail "NETWORK_MODE=bridge requires BRIDGE_IF=..."
  [[ -d "/sys/class/net/$BRIDGE_IF" ]] || fail "Bridge interface not found: $BRIDGE_IF"
fi

if [[ -n "$GPU_VIDEO_PCI" && -z "$GPU_AUDIO_PCI" ]]; then
  echo "WARN: GPU_VIDEO_PCI is set without GPU_AUDIO_PCI. Continuing with video function only."
fi

if [[ ! -d "$POOL_DIR" ]]; then
  info "Creating pool directory: $POOL_DIR"
  sudo mkdir -p "$POOL_DIR"
  sudo chown "$(id -u)":"$(id -g)" "$POOL_DIR" || true
fi

info "Ensuring libvirt is running"
sudo systemctl enable --now libvirtd

VIRSH=(virsh --connect "$LIBVIRT_URI")
VIRT_INSTALL_CONNECT=(--connect "$LIBVIRT_URI")

if [[ "$NETWORK_MODE" == "nat" ]]; then
  info "Ensuring libvirt default NAT network exists and is running"
  if ! "${VIRSH[@]}" net-info default >/dev/null 2>&1; then
    fail "libvirt network 'default' does not exist under $LIBVIRT_URI. Create it first or use NETWORK_MODE=bridge."
  fi
  if ! "${VIRSH[@]}" net-list --name | grep -Fxq default; then
    "${VIRSH[@]}" net-start default
  fi
  "${VIRSH[@]}" net-autostart default >/dev/null
fi

if "${VIRSH[@]}" dominfo "$VM_NAME" >/dev/null 2>&1; then
  fail "A VM named '$VM_NAME' already exists in libvirt"
fi

if [[ ! -f "$DISK_PATH" ]]; then
  info "Creating disk: $DISK_PATH (${DISK_GB}G)"
  qemu-img create -f qcow2 "$DISK_PATH" "${DISK_GB}G"
else
  fail "Disk already exists: $DISK_PATH"
fi

NET_ARG=(--network "network=default,model=virtio")
if [[ "$NETWORK_MODE" == "bridge" ]]; then
  NET_ARG=(--network "bridge=${BRIDGE_IF},model=virtio")
fi

TPM_ARGS=(--tpm backend.type=emulator,backend.version=2.0,model=tpm-crb)
GRAPHICS_ARGS=(--graphics spice --video qxl)
PASSTHROUGH_ARGS=()

if [[ -n "$GPU_VIDEO_PCI" ]]; then
  GRAPHICS_ARGS=(--graphics none --video none)
  PASSTHROUGH_ARGS+=(--hostdev "$GPU_VIDEO_PCI")
fi

if [[ -n "$GPU_AUDIO_PCI" ]]; then
  PASSTHROUGH_ARGS+=(--hostdev "$GPU_AUDIO_PCI")
fi

info "Creating VM '$VM_NAME'"
set -x
virt-install \
  "${VIRT_INSTALL_CONNECT[@]}" \
  --name "$VM_NAME" \
  --machine q35 \
  --virt-type kvm \
  --cpu "$CPU_MODE" \
  --vcpus "$VCPUS" \
  --memory "$RAM_MB" \
  --boot loader=/usr/share/edk2-ovmf/x64/OVMF_CODE.4m.fd,loader.readonly=yes,loader.type=pflash,nvram.template=/usr/share/edk2-ovmf/x64/OVMF_VARS.4m.fd \
  --disk "path=$DISK_PATH,format=qcow2,bus=${EXTRA_DISK_BUS}" \
  --disk "path=$WIN_ISO,device=cdrom,bus=sata" \
  --disk "path=$VIRTIO_ISO,device=cdrom,bus=sata" \
  "${NET_ARG[@]}" \
  "${TPM_ARGS[@]}" \
  "${GRAPHICS_ARGS[@]}" \
  "${PASSTHROUGH_ARGS[@]}" \
  --features kvm_hidden=on,smm=on \
  --clock hypervclock_present=yes \
  --os-variant "$OS_VARIANT" \
  --channel spicevmc \
  --sound ich9 \
  --rng /dev/urandom \
  --controller type=usb,model=qemu-xhci \
  --input type=tablet,bus=usb \
  --input type=keyboard,bus=usb \
  --noautoconsole
set +x

cat <<EOF

VM created successfully.

Summary:
  Name:        $VM_NAME
  Disk:        $DISK_PATH
  Memory:      $RAM_MB MB
  vCPUs:       $VCPUS
  Network:     $([[ "$NETWORK_MODE" == "bridge" ]] && echo "bridge ($BRIDGE_IF)" || echo "nat (libvirt default)")
  GPU PT:      $([[ -n "$GPU_VIDEO_PCI" ]] && echo "yes" || echo "no")

Useful commands:
  virsh --connect $LIBVIRT_URI start $VM_NAME
  virt-manager --connect $LIBVIRT_URI
  virsh --connect $LIBVIRT_URI domifaddr $VM_NAME

Notes:
- NAT is the sane default on a Wi-Fi-only host.
- A normal Linux bridge over wlan0 usually does not work in station/managed mode.
- If you want bridged networking later, create a real bridge like br0 over Ethernet, then run:
    NETWORK_MODE=bridge BRIDGE_IF=br0 ./scripts/VMs/create_windows_vm.sh
- For GPU passthrough, run with:
    GPU_VIDEO_PCI=0000:01:00.0 GPU_AUDIO_PCI=0000:01:00.1 ./scripts/VMs/create_windows_vm.sh
EOF
