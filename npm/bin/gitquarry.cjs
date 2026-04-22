#!/usr/bin/env node

const { spawn } = require("node:child_process");
const { ensureInstalled, getBinaryPath } = require("../lib/install.cjs");

async function main() {
  await ensureInstalled({ quiet: false });

  const child = spawn(getBinaryPath(), process.argv.slice(2), {
    stdio: "inherit",
  });

  child.on("exit", (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
      return;
    }

    process.exit(code ?? 0);
  });

  child.on("error", (error) => {
    console.error(`failed to start gitquarry: ${error.message}`);
    process.exit(1);
  });
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
