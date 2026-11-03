# Linux VM for testing minirunc (namespaces and mounts only exist on Linux).
#
#   vagrant up                 # first time: creates the VM and sets it up
#   vagrant ssh -c test-create # build and run `minirunc create`
#   vagrant destroy -f         # delete the VM

Vagrant.configure("2") do |config|
  config.vm.box = "bento/ubuntu-24.04"
  config.vm.hostname = "minirunc"

  config.vm.provider "virtualbox" do |vb|
    vb.cpus = 2
    vb.memory = 2048
  end

  # The project is shared at /vagrant (the default).

  # System packages.
  config.vm.provision "shell", inline: <<~SHELL
    set -e
    apt-get update -q
    apt-get install -yq build-essential curl
  SHELL

  # Rust, for the vagrant user.
  config.vm.provision "shell", privileged: false, inline: <<~SHELL
    set -e
    if ! command -v cargo >/dev/null 2>&1 && [ ! -x "$HOME/.cargo/bin/cargo" ]; then
      curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
    fi
  SHELL

  # A test bundle: Alpine rootfs + config.json, at /home/vagrant/bundle.
  config.vm.provision "shell", privileged: false, inline: <<~SHELL
    set -e
    BUNDLE=$HOME/bundle
    mkdir -p "$BUNDLE/rootfs"
    if [ ! -e "$BUNDLE/rootfs/bin/sh" ]; then
      ARCH=$(uname -m)
      curl -sSfL "https://dl-cdn.alpinelinux.org/alpine/v3.20/releases/$ARCH/alpine-minirootfs-3.20.3-$ARCH.tar.gz" \
        | tar -xz -C "$BUNDLE/rootfs"
    fi
    cat > "$BUNDLE/config.json" <<'JSON'
    {
      "hostname": "box",
      "process": {
        "args": ["/bin/sh"],
        "env": ["PATH=/usr/sbin:/usr/bin:/sbin:/bin", "TERM=xterm"],
        "cwd": "/"
      },
      "root": { "path": "rootfs" },
      "linux": {
        "namespaces": [
          { "type": "pid" }, { "type": "mount" }, { "type": "uts" },
          { "type": "ipc" }
        ]
      }
    }
  JSON
  SHELL

  # `test-create`: builds minirunc and runs `create` against the bundle.
  # Builds into ~/target so it doesn't clash with the Mac's ./target.
  config.vm.provision "shell", inline: <<~SHELL
    cat > /usr/local/bin/test-create <<'SCRIPT'
    #!/bin/bash
    set -e
    source "$HOME/.cargo/env"
    export CARGO_TARGET_DIR=$HOME/target
    cd /vagrant
    cargo build
    BIN=$CARGO_TARGET_DIR/debug/minirunc
    ID=${1:-test}

    sudo rm -rf "/run/minirunc/$ID"   # start clean
    echo "== create $ID"
    sudo "$BIN" create "$ID" --bundle "$HOME/bundle"
    echo "== state dir"
    sudo ls -la "/run/minirunc/$ID"
    echo "== create again (should fail: already exists)"
    sudo "$BIN" create "$ID" --bundle "$HOME/bundle" || true
    echo "== create without root (should fail)"
    "$BIN" create other --bundle "$HOME/bundle" || true
    SCRIPT
    chmod +x /usr/local/bin/test-create
  SHELL
end
