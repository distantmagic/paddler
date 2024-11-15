declare const MemoryTests_base: import("../../../core/constructor").Constructor<import("../../cases/controller_test_case").ControllerTests<import("../../..").Controller<Element>>>;
export default class MemoryTests extends MemoryTests_base {
    controllerElement: Element;
    setup(): Promise<void>;
    fixtureHTML: string;
    "test removing a controller clears dangling eventListeners"(): Promise<void>;
}
export {};
