import Subscriber from "@mercury-pubsub/subscriber";

describe("subscriber", function () {
  it("is defined", function () {
    const subscriber = new Subscriber("url", "key");
    expect(subscriber).toBeDefined();
  });
});
