import { fireEvent, render, screen } from "@testing-library/react";
import renderer from "react-test-renderer";

import ThemeMenu from "../../src/components/ThemeMenu";

const setTheme = jest.fn();
jest.mock("next-themes", () => ({
  useTheme: jest.fn(() => ({ setTheme })),
}));

describe("ThemeMenu", function () {
  it("renders unchanged", function () {
    const tree = renderer.create(<ThemeMenu />).toJSON();
    expect(tree).toMatchSnapshot();
  });

  it("opens correctly", function () {
    render(<ThemeMenu />);
    fireEvent.click(screen.getByText(/theme/i));
    expect(screen.getByText(/light/i)).toBeDefined();
    expect(screen.getByText(/dark/i)).toBeDefined();
    expect(screen.getByText(/system/i)).toBeDefined();
  });

  it("changes themes correctly", function () {
    render(<ThemeMenu />);
    const menuButton = screen.getByText(/theme/i);

    fireEvent.click(menuButton);
    fireEvent.click(screen.getByText(/light/i));
    expect(setTheme).toHaveBeenCalledTimes(1);
    expect(setTheme).toHaveBeenLastCalledWith("light");

    fireEvent.click(menuButton);
    fireEvent.click(screen.getByText(/dark/i));
    expect(setTheme).toHaveBeenCalledTimes(2);
    expect(setTheme).toHaveBeenLastCalledWith("dark");

    fireEvent.click(menuButton);
    fireEvent.click(screen.getByText(/system/i));
    expect(setTheme).toHaveBeenCalledTimes(3);
    expect(setTheme).toHaveBeenLastCalledWith("system");
  });

  it("styles menu items correctly on hover", function () {
    render(<ThemeMenu />);
    fireEvent.click(screen.getByText(/theme/i));
    const lightMenuItem = screen.getByText(/light/i);
    const darkMenuItem = screen.getByText(/dark/i);
    const systemMenuItem = screen.getByText(/system/i);

    fireEvent.focus(lightMenuItem);
    expect(lightMenuItem.classList).toContain("bg-violet-500");
    expect(darkMenuItem.classList).not.toContain("bg-violet-500");
    expect(systemMenuItem.classList).not.toContain("bg-violet-500");

    fireEvent.focus(darkMenuItem);
    expect(lightMenuItem.classList).not.toContain("bg-violet-500");
    expect(darkMenuItem.classList).toContain("bg-violet-500");
    expect(systemMenuItem.classList).not.toContain("bg-violet-500");

    fireEvent.focus(systemMenuItem);
    expect(lightMenuItem.classList).not.toContain("bg-violet-500");
    expect(darkMenuItem.classList).not.toContain("bg-violet-500");
    expect(systemMenuItem.classList).toContain("bg-violet-500");
  });
});
