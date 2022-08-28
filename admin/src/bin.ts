import { createInterface, cursorTo, moveCursor } from "node:readline";

import "dotenv/config";
import type { Argv } from "yargs";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import { z } from "zod";

import Mercury, { keyTypes } from "./index.js";

const env = z
  .object({
    URL: z.string().url(),
    NAME: z.string(),
    PASSWORD: z.optional(z.string()),
  })
  .parse(process.env);

if (env.PASSWORD === undefined) {
  const rl = createInterface(process.stdin);
  process.stdout.write("password:");
  env.PASSWORD = await new Promise<string>(function (resolve) {
    let answer: string;
    rl.on("line", function (input) {
      answer = input;
      rl.close();
    });
    rl.on("close", function () {
      resolve(answer);
    });
  });
  moveCursor(process.stdout, 0, -1);
  process.stdout.write(" ".repeat(env.PASSWORD.length + 9));
  cursorTo(process.stdout, 0);
}

const mercury = new Mercury(env.URL, env.NAME, env.PASSWORD);

const Uuid = z.string().uuid();

function userCommands(yargs: Argv) {
  return (
    yargs
      // user.list
      .command("list", "list users", async function () {
        console.log(await mercury.user.list());
      })
      // user.create
      .command(
        "create [name] [password]",
        "create a user",
        function (yargs) {
          return yargs
            .positional("name", { type: "string" })
            .positional("password", { type: "string" })
            .demandOption(["name", "password"]);
        },
        async function (argv) {
          console.log(await mercury.user.create(argv.name, argv.password));
        }
      )
      // user.rename
      .command(
        "rename [name]",
        "rename self",
        function (yargs) {
          return yargs.positional("name", { type: "string" }).demandOption("name");
        },
        async function (argv) {
          console.log(await mercury.user.rename(argv.name));
        }
      )
      // user.delete
      .command(
        "delete [id]",
        "delete a user",
        function (yargs) {
          return yargs
            .positional("id", {
              coerce(arg) {
                return Uuid.parse(arg);
              },
            })
            .demandOption("id");
        },
        async function (argv) {
          await mercury.user.delete(argv.id);
        }
      )
  );
}

function channelCommands(yargs: Argv) {
  return (
    yargs
      // channel.list
      .command("list", "list channels", async function () {
        console.log(await mercury.channel.list());
      })
      // channel.create
      .command(
        "create [name] [schema]",
        "create a channel",
        function (yargs) {
          return yargs
            .positional("name", { type: "string" })
            .positional("schema", {
              coerce(arg) {
                return z.record(z.unknown()).parse(arg);
              },
            })
            .demandOption(["name", "schema"]);
        },
        async function (argv) {
          console.log(await mercury.channel.create(argv.name, argv.schema));
        }
      )
      // channel.delete
      .command(
        "delete [id]",
        "delete a channel",
        function (yargs) {
          return yargs
            .positional("id", {
              coerce(arg) {
                return Uuid.parse(arg);
              },
            })
            .demandOption("id");
        },
        async function (argv) {
          await mercury.channel.delete(argv.id);
        }
      )
  );
}

function keyCommands(yargs: Argv) {
  return (
    yargs
      // key.list
      .command("list", "list keys", async function () {
        console.log(await mercury.key.list());
      })
      // key.create
      .command(
        "create [type] [channels..]",
        "create a key",
        function (yargs) {
          return yargs
            .positional("type", { choices: keyTypes })
            .positional("channels", {
              coerce(arg) {
                return z.array(Uuid).min(1).parse(arg);
              },
            })
            .demandOption(["type", "channels"]);
        },
        async function (argv) {
          console.log(await mercury.key.create(argv.type, argv.channels));
        }
      )
      // key.listChannels
      .command(
        "list-channels [id]",
        "list the channels a key authorizes",
        function (yargs) {
          return yargs
            .positional("id", {
              coerce(arg) {
                return Uuid.parse(arg);
              },
            })
            .demandOption("id");
        },
        async function (argv) {
          console.log(await mercury.key.listChannels(argv.id));
        }
      )
      // key.delete
      .command(
        "delete [id]",
        "delete a key",
        function (yargs) {
          return yargs
            .positional("id", {
              coerce(arg) {
                return Uuid.parse(arg);
              },
            })
            .demandOption("id");
        },
        async function (argv) {
          await mercury.key.delete(argv.id);
        }
      )
  );
}

await yargs(hideBin(process.argv))
  .command("users", "manage users", userCommands)
  .command("channels", "manage channels", channelCommands)
  .command("keys", "manage keys", keyCommands)
  .parse();
