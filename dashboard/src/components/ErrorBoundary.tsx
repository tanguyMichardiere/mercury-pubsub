import type { ErrorInfo, ReactNode } from "react";
import { Component } from "react";

import Link from "next/link";

type Props = {
  children: ReactNode;
};

type State = {
  error: unknown;
};

export default class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { error: undefined };
  }

  static getDerivedStateFromError(error: unknown): State {
    return { error };
  }

  componentDidCatch(error: unknown, errorInfo: ErrorInfo): void {
    console.error({ error, errorInfo });
  }

  reload(this: void): void {
    location.reload();
  }

  render(): JSX.Element {
    if (this.state.error !== undefined) {
      return (
        <div className="flex h-full flex-col items-center justify-center">
          <h2>An expected error has occurred, it has been reported</h2>
          <Link href="/">Home Page</Link>
          <button onClick={this.reload} type="button">
            Reload
          </button>
        </div>
      );
    }

    return <>{this.props.children}</>;
  }
}
