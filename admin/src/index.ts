import { z } from "zod";

const Uuid = z.string().uuid();

const User = z.object({
  id: Uuid,
  name: z.string(),
});
type User = z.infer<typeof User>;

const Channel = z.object({
  id: Uuid,
  name: z.string(),
  // TODO: JSON-schema type
  schema: z.record(z.unknown()),
});
type Channel = z.infer<typeof Channel>;

export const keyTypes = ["publisher", "subscriber"] as const;
const KeyType = z.enum(keyTypes);
type KeyType = z.infer<typeof KeyType>;

const Key = z.object({
  id: Uuid,
  type: KeyType,
});
type Key = z.infer<typeof Key>;

export default class Mercury {
  #url: string;
  #name: string;
  #password: string;

  #user: UserDelegate;
  #channel: ChannelDelegate;
  #key: KeyDelegate;

  constructor(url: string, name: string, password: string) {
    if (password === "mercury") {
      console.warn("WARNING: you are using the default password for the admin user.");
      console.warn("WARNING: you should change your password.");
    }
    this.#url = url;
    this.#name = name;
    this.#password = password;

    const authorizationHeader = this.#authorizationHeader();

    this.#user = new UserDelegate(
      url,
      authorizationHeader,
      this.#updateName.bind(this),
      this.#updatePassword.bind(this)
    );
    this.#channel = new ChannelDelegate(url, authorizationHeader);
    this.#key = new KeyDelegate(url, authorizationHeader);
  }

  #authorizationHeader(): string {
    return `Basic ${Buffer.from(`${this.#name}:${this.#password}`, "utf8").toString("base64")}`;
  }

  #updateName(name: string) {
    this.#name = name;
    const authorizationHeader = this.#authorizationHeader();
    this.#user = new UserDelegate(
      this.#url,
      authorizationHeader,
      this.#updateName.bind(this),
      this.#updatePassword.bind(this)
    );
    this.#channel = new ChannelDelegate(this.#url, authorizationHeader);
    this.#key = new KeyDelegate(this.#url, authorizationHeader);
  }

  #updatePassword(password: string) {
    this.#password = password;
    const authorizationHeader = this.#authorizationHeader();
    this.#user = new UserDelegate(
      this.#url,
      authorizationHeader,
      this.#updateName.bind(this),
      this.#updatePassword.bind(this)
    );
    this.#channel = new ChannelDelegate(this.#url, authorizationHeader);
    this.#key = new KeyDelegate(this.#url, authorizationHeader);
  }

  get user(): UserDelegate {
    return this.#user;
  }

  get channel(): ChannelDelegate {
    return this.#channel;
  }

  get key(): KeyDelegate {
    return this.#key;
  }
}

class UserDelegate {
  #url: string;
  #authorizationHeader: string;

  #updateName: (name: string) => void;
  #updatePassword: (password: string) => void;

  constructor(
    url: string,
    authorizationHeader: string,
    updateName: (name: string) => void,
    updatePassword: (password: string) => void
  ) {
    this.#url = url;
    this.#authorizationHeader = authorizationHeader;

    this.#updateName = updateName;
    this.#updatePassword = updatePassword;
  }

  async list(): Promise<Array<User>> {
    const url = new URL("/api/users", this.#url);
    const response = await fetch(url.href, {
      headers: { Authorization: this.#authorizationHeader },
    });
    if (!response.ok) throw new Error(await response.text());
    return z.array(User).parse(await response.json());
  }

  async create(name: string, password: string): Promise<User> {
    const url = new URL("/api/users", this.#url);
    const response = await fetch(url.href, {
      method: "POST",
      headers: {
        Authorization: this.#authorizationHeader,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ name, password }),
    });
    if (!response.ok) throw new Error(await response.text());
    return User.parse(await response.json());
  }

  async rename(name: string): Promise<void> {
    const url = new URL("/api/users/rename", this.#url);
    const response = await fetch(url.href, {
      method: "PATCH",
      headers: {
        Authorization: this.#authorizationHeader,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ name }),
    });
    if (!response.ok) throw new Error(await response.text());
    this.#updateName(name);
  }

  async changePassword(password: string): Promise<void> {
    const url = new URL("/api/users/change-password", this.#url);
    const response = await fetch(url.href, {
      method: "PATCH",
      headers: {
        Authorization: this.#authorizationHeader,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ password }),
    });
    if (!response.ok) throw new Error(await response.text());
    this.#updatePassword(password);
  }

  async delete(id: string): Promise<void> {
    const url = new URL(`/api/users/${id}`, this.#url);
    const response = await fetch(url.href, {
      method: "DELETE",
      headers: { Authorization: this.#authorizationHeader },
    });
    if (!response.ok) throw new Error(await response.text());
  }
}

class ChannelDelegate {
  #url: string;
  #authorizationHeader: string;

  constructor(url: string, authorizationHeader: string) {
    this.#url = url;
    this.#authorizationHeader = authorizationHeader;
  }

  async list(): Promise<Array<Channel>> {
    const url = new URL("/api/channels", this.#url);
    const response = await fetch(url.href, {
      headers: { Authorization: this.#authorizationHeader },
    });
    if (!response.ok) throw new Error(await response.text());
    return z.array(Channel).parse(await response.json());
  }

  async create(name: string, schema: Record<string, unknown>): Promise<Channel> {
    const url = new URL("/api/channels", this.#url);
    const response = await fetch(url.href, {
      method: "POST",
      headers: {
        Authorization: this.#authorizationHeader,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ name, schema }),
    });
    if (!response.ok) throw new Error(await response.text());
    return Channel.parse(await response.json());
  }

  async delete(id: string): Promise<void> {
    const url = new URL(`/api/channels/${id}`, this.#url);
    const response = await fetch(url.href, {
      method: "DELETE",
      headers: { Authorization: this.#authorizationHeader },
    });
    if (!response.ok) throw new Error(await response.text());
  }
}

class KeyDelegate {
  #url: string;
  #authorizationHeader: string;

  constructor(url: string, authorizationHeader: string) {
    this.#url = url;
    this.#authorizationHeader = authorizationHeader;
  }

  async list(): Promise<Array<Key>> {
    const url = new URL("/api/keys", this.#url);
    const response = await fetch(url.href, {
      headers: { Authorization: this.#authorizationHeader },
    });
    if (!response.ok) throw new Error(await response.text());
    return z.array(Key).parse(await response.json());
  }

  async create(type: KeyType, channels: Array<string>): Promise<string> {
    const url = new URL("/api/keys", this.#url);
    const response = await fetch(url.href, {
      method: "POST",
      headers: {
        Authorization: this.#authorizationHeader,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ type, channels }),
    });
    if (!response.ok) throw new Error(await response.text());
    return await response.text();
  }

  async listChannels(id: string): Promise<Array<Channel>> {
    const url = new URL(`/api/keys/${id}`, this.#url);
    const response = await fetch(url.href, {
      headers: { Authorization: this.#authorizationHeader },
    });
    if (!response.ok) throw new Error(await response.text());
    return z.array(Channel).parse(await response.json());
  }

  async delete(id: string): Promise<void> {
    const url = new URL(`/api/keys/${id}`, this.#url);
    const response = await fetch(url.href, {
      method: "DELETE",
      headers: { Authorization: this.#authorizationHeader },
    });
    if (!response.ok) throw new Error(await response.text());
  }
}
