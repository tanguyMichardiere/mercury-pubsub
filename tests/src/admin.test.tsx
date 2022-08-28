import Mercury from "@mercury-pubsub/admin";

describe("admin", function () {
  it("is defined", function () {
    const mercury = new Mercury("url", "name", "password");
    expect(mercury).toBeDefined();
  });
});
