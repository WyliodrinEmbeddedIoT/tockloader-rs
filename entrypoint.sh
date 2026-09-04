#!/usr/bin/env bash

  set -euo pipefail

  TOCK_HASH="236be95a8a7747b91ca855ffe245ae475587258d"

  : "${HILLTOP_PROBE_SELECTOR:?HILLTOP_PROBE_SELECTOR was not provided by the runner}"

  HILLTOP_PROBE_SERIAL="${HILLTOP_PROBE_SELECTOR##*:}"

  echo "-=-= Start test pipeline =-=-"
  echo "Probe selector: ${HILLTOP_PROBE_SELECTOR}"
  echo "Probe serial: ${HILLTOP_PROBE_SERIAL}"

  echo " [0/6] Fully erase the board"

  probe-rs erase \
      --chip nrf52840_xxAA \
      --probe "$HILLTOP_PROBE_SELECTOR" \
      2>&1

  echo " [1/6] Download Tock"

  TEMP_DIR="$(mktemp -d)"
  trap 'rm -rf "$TEMP_DIR"' EXIT

  cd "$TEMP_DIR"

  git clone \
      --no-checkout \
      --filter=blob:none \
      https://github.com/tock/tock.git \
      tock \
      2>&1

  cd tock
  git checkout "$TOCK_HASH" 2>&1

  echo "Checked out Tock commit ${TOCK_HASH}"

  OPENOCD_CONFIG="/opt/openocd/share/openocd/scripts/board/jtag/nrf52840dk.cfg"

    if [[ ! -f "$OPENOCD_CONFIG" ]]; then
        echo "OpenOCD configuration not found: $OPENOCD_CONFIG" >&2
        exit 1
    fi

    if ! grep -q "adapter serial" "$OPENOCD_CONFIG"; then
        sed -i \
            "/source.*interface\\/jlink.cfg/a adapter serial ${HILLTOP_PROBE_SERIAL}" \
            "$OPENOCD_CONFIG"
fi

  echo " [2/6] Build and flash Tock"

  cd "${TEMP_DIR}/tock/boards/nordic/nrf52840dk"

  make flash-openocd 2>&1

  echo "Flashed Tock to nrf52840dk"

  echo " [3/6] Prepare board with tockloader"

  cd /workspace/test_data

  tockloader install \
        --board nrf52dk \
        --openocd \
        --openocd-board jtag/nrf52840dk.cfg \
        --openocd-commands "adapter serial ${HILLTOP_PROBE_SERIAL}" \
        ./c_hello.tab \
        2>&1

  cd /workspace

  echo " [4/6] Build tockloader-rs"

  apt-get update
  apt-get install -y --no-install-recommends libudev-dev

  cargo build --release -p tockloader 2>&1

  echo "Built tockloader-rs"

  echo " [5/6] Test tockloader-rs"

  LIST_OUTPUT="$(
      cargo run --release -p tockloader -- \
          list \
          --board nrf52840dk \
          --probe "$HILLTOP_PROBE_SELECTOR"
  )"

  echo "$LIST_OUTPUT" > list_output.txt

  if [[ "$LIST_OUTPUT" != *"c_hello"* ]]; then
      echo "Error: 'list' command did not find expected application 'c_hello'"
      echo "Output was:"
      echo "$LIST_OUTPUT"
      exit 1
  fi

  echo "Success: 'list' command contains expected application 'c_hello'"

  echo " [6/6] cargo test -p tockloader-lib"

  cargo test -p tockloader-lib --lib 2>&1
  cargo test -p tockloader-lib --test testuite -- --ignored 2>&1

  echo "-=-= End test pipeline =-=-"
