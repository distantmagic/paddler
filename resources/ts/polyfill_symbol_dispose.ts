if (!Symbol.dispose) {
  console.warn("[polyfill] Symbol.dispose");

  Object.defineProperty(Symbol, "dispose", {
    configurable: false,
    enumerable: false,
    value: Symbol("Symbol.dispose"),
    writable: false,
  });
}

if (!Symbol.asyncDispose) {
  console.warn("[polyfill] Symbol.asyncDispose");

  Object.defineProperty(Symbol, "asyncDispose", {
    configurable: false,
    enumerable: false,
    value: Symbol("Symbol.asyncDispose"),
    writable: false,
  });
}
