import { Controller } from "@hotwired/stimulus";

import { stimulus } from "./stimulus";

@stimulus("refresh_body")
export class controller_refresh_body extends Controller<HTMLElement> {
  public connect(): void {
    setInterval(function () {
      globalThis.Turbo.visit(window.location, {
        action: "replace",
      });
    }, 1000);
  }
}
