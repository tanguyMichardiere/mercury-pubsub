import { useState } from "react";

import type { NextPage } from "next";

import { useSubscribe } from "@mercury-pubsub/subscriber/react";

const Home: NextPage = () => {
  const [messages, setMessages] = useState<Array<[Date, string]>>([]);

  useSubscribe("messages", {
    async onopen(response) {
      console.log(response);
    },
    ondata(data) {
      if (typeof data === "string") {
        setMessages((messages) => [...messages, [new Date(), data]]);
      }
    },
  });

  const [message, setMessage] = useState("");

  return (
    <div>
      {messages.map(([sentAt, message]) => (
        <p key={sentAt.getTime()}>
          {sentAt.toISOString()} {message}
        </p>
      ))}
      <form
        onSubmit={function (event) {
          event.preventDefault();
          fetch("/api/send", { method: "POST", body: message })
            .then((response) => {
              setMessage("");
              return response.json();
            })
            .then((count) => console.log(`seen by ${count} people`));
        }}
      >
        <input
          type="text"
          value={message}
          onChange={function (event) {
            setMessage(event.target.value);
          }}
        />
        <button type="submit" />
      </form>
    </div>
  );
};

export default Home;
