#!/usr/bin/env npx zx
import { $ } from "zx";

await $`npx -y npm-check-updates -u`;
await $`npm install`;
await $`npm update`;
