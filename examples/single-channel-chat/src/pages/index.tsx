import { useState } from "react";

import type { NextPage } from "next";

import { useSubscribe } from "@mercury-pubsub/subscriber/react";

const Home: NextPage = () => {
  const [messages, setMessages] = useState<Array<string>>([]);

  useSubscribe("messages", {
    ondata(data) {
      if (typeof data === "string") {
        setMessages((messages) => [...messages, data]);
      }
    },
  });

  const [message, setMessage] = useState("");

  return (
    <div>
      {messages.map((message) => (
        <p key={message}>{message}</p>
      ))}
      <form
        onSubmit={function (event) {
          event.preventDefault();
          fetch("/api/send", { method: "POST", body: message })
            .then((response) => {
              setMessage("");
              return response.json();
            })
            .then((count) => console.log(`seen by ${count} others`));
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
