import { fireEvent, render, screen } from "@testing-library/react";
import renderer from "react-test-renderer";

import ErrorBoundary from "../../src/components/ErrorBoundary";

console.error = jest.fn();

const reload = jest.fn();
const location = window.location;
// @ts-expect-error necessary for mocking window.location methods
delete window.location;
window.location = { ...location, reload };

function ErrorThrower(): JSX.Element {
  throw Error();
}

describe("ErrorBoundary", function () {
  it("renders unchanged", function () {
    const tree = renderer
      .create(
        <ErrorBoundary>
          <ErrorThrower />
        </ErrorBoundary>
      )
      .toJSON();
    expect(tree).toMatchSnapshot();
  });

  it("allows to reload the page", function () {
    render(
      <ErrorBoundary>
        <ErrorThrower />
      </ErrorBoundary>
    );
    fireEvent.click(screen.getByText(/reload/i));
    expect(reload).toHaveBeenCalledTimes(1);
  });
});
