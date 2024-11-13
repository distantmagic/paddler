import type { Application, Controller } from "@hotwired/stimulus";

declare global {
  interface Window {
    stimulusApplication: undefined | Application;
  }
}

export function stimulus(controllerName: string) {
  return function register(
    constructor: typeof Controller<HTMLElement>,
    retries: number = 0,
  ): void {
    const stimulusApplication = window.stimulusApplication;

    if (retries > 10) {
      console.error(
        "[stimulus]",
        "too many attempts;",
        "giving up on trying to register controller:",
        controllerName,
      );

      return;
    }

    if (!stimulusApplication) {
      console.warn(
        "[stimulus]",
        "stimulus is not yet loaded;",
        `unable to register controller: ${controllerName};`,
        "scheduled to retry after 100ms",
      );

      setTimeout(register.bind(null, constructor, retries + 1), 100);

      return;
    }

    if (retries > 0) {
      console.info(
        "[stimulus] controller registered after retrying:",
        controllerName,
      );
    } else {
      console.info("[stimulus] controller registered:", controllerName);
    }

    stimulusApplication.register(controllerName, constructor);
  };
}
