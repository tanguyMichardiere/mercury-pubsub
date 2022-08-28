import Publisher from "@mercury-pubsub/publisher";

describe("publisher", function () {
  it("is defined", function () {
    const publisher = new Publisher("url", "key");
    expect(publisher).toBeDefined();
  });
});
