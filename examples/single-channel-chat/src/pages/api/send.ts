import { NextRequest } from "next/server";

import Publisher from "@mercury-pubsub/publisher";

export const config = {
  runtime: "experimental-edge",
};

const publisher = new Publisher(
  process.env.NEXT_PUBLIC_MERCURY_URL,
  process.env.MERCURY_PUBLISHER_KEY
);

export default async function handler(req: NextRequest) {
  const count = await publisher.publish("messages", await req.text());
  return new Response(count.toString());
}
