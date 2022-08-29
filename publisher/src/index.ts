type Channels = Record<string, unknown>;

export default class Publisher {
  #url: string;
  #key: string;

  constructor(url: string, key: string) {
    this.#url = url;
    this.#key = key;
  }

  async publish<C extends keyof Channels>(channel: C, data: Channels[C]): Promise<number> {
    const url = new URL(`/sse/${channel}`, this.#url);
    const response = await fetch(url.href, {
      method: "POST",
      body: JSON.stringify(data),
      headers: {
        Authorization: `Bearer ${this.#key}`,
        "Content-Type": "application/json",
      },
    });
    if (!response.ok) throw new Error(await response.text());
    return parseInt(await response.text());
  }
}
