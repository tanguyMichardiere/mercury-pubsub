// Next.js API route support: https://nextjs.org/docs/api-routes/introduction
import type { NextApiRequest, NextApiResponse } from "next";

import Publisher from "@mercury-pubsub/publisher";

const publisher = new Publisher(
  process.env.NEXT_PUBLIC_MERCURY_URL,
  process.env.MERCURY_PUBLISHER_KEY
);

export default async function handler(req: NextApiRequest, res: NextApiResponse<number>) {
  const count = await publisher.publish("messages", req.body);
  res.status(200).json(count);
}
