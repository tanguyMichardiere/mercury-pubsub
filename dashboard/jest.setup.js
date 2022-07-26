import { jest } from "@jest/globals";
import "@testing-library/jest-dom/extend-expect";

jest
  .spyOn(Date.prototype, "toLocaleDateString")
  .mockImplementation(() => "Date.toLocaleDateString");
jest.spyOn(Date.prototype, "toLocaleString").mockImplementation(() => "Date.toLocaleString");
jest
  .spyOn(Date.prototype, "toLocaleTimeString")
  .mockImplementation(() => "Date.toLocaleTimeString");

jest.mock("next/router", () => ({
  ...jest.requireActual("next/router"),
  useRouter: jest.fn(() => ({})),
}));

jest.mock("next/link", () => ({ children, passHref, ...props }) => <a {...props}>{children}</a>);

jest.mock("next/future/image", () => ({ children, passHref, ...props }) => (
  <img {...props}>{children}</img>
));

const Div = ({ children }) => <div>{children}</div>;
const Dialog = jest.fn(Div);
Dialog.Overlay = jest.fn(Div);
Dialog.Title = jest.fn(Div);

jest.mock("@headlessui/react", () => ({
  ...jest.requireActual("@headlessui/react"),
  Dialog,
}));
