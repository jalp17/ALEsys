# Firecracker Sandbox (On-Prem Enterprise)

## Overview
Firecracker microVMs provide hardware-level isolation for untrusted code execution.
Used as the highest-security tier in ALEsys `CodeSandbox`.

## Prerequisites
- Linux host with KVM support (`/dev/kvm`)
- Firecracker binary v1.6+
- `containerd` + `firecracker-containerd` shim
- CNI plugins (bridge, loopback, host-local)

## Architecture
```
ALEsys Core
    │
    ▼
FirecrackerSandbox (implements CodeSandbox trait)
    │
    ├── firecracker-containerd (shim)
    │       │
    │       ▼
    │   Firecracker microVM (guest kernel, <5MB overhead)
    │       │
    │       └── container (python:3.11 / custom image)
    │
    └── CNI network (none by default, configurable)
```

## Setup
```bash
# 1. Install Firecracker
curl -L https://github.com/firecracker-microvm/firecracker/releases/download/v1.6.0/firecracker-v1.6.0-x86_64.tgz | tar xz
sudo mv firecracker-v1.6.0-x86_64 /usr/local/bin/firecracker

# 2. Install firecracker-containerd
git clone https://github.com/firecracker-microvm/firecracker-containerd
cd firecracker-containerd && make && sudo make install

# 3. Configure containerd
sudo cp examples/config.toml /etc/containerd/config.toml
sudo systemctl restart containerd

# 4. Pull rootfs
sudo ctr image pull docker.io/library/python:3.11-slim
```

## Configuration (ALEsys)
Set env vars:
```
SANDBOX_BACKEND=firecracker
FIRECRACKER_KERNEL=/var/lib/firecracker/vmlinux
FIRECRACKER_ROOTFS=/var/lib/firecracker/rootfs.ext4
```

## Security
- Each sandbox runs in its own guest kernel (no shared kernel attack surface)
- Network disabled by default (`--network=none`)
- Read-only rootfs
- CPU/memory limits enforced by Firecracker
- Max 24h session lifetime

## Status
Documentation + configuration only. Full `FirecrackerSandbox` implementation
requires KVM-capable host. For dev/CI, use `DockerSandbox` (gVisor).
For production SaaS, use `E2BSandbox` (cloud Firecracker).
