import { TestCase } from "./test_case";
interface TriggerEventOptions {
    bubbles?: boolean;
    setDefaultPrevented?: boolean;
}
export declare class DOMTestCase extends TestCase {
    fixtureSelector: string;
    fixtureHTML: string;
    runTest(testName: string): Promise<void>;
    renderFixture(fixtureHTML?: string): Promise<any>;
    get fixtureElement(): Element;
    triggerEvent(selectorOrTarget: string | EventTarget, type: string, options?: TriggerEventOptions): Promise<Event>;
    triggerMouseEvent(selectorOrTarget: string | EventTarget, type: string, options?: MouseEventInit): Promise<MouseEvent>;
    triggerKeyboardEvent(selectorOrTarget: string | EventTarget, type: string, options?: KeyboardEventInit): Promise<KeyboardEvent>;
    setAttribute(selectorOrElement: string | Element, name: string, value: string): Promise<void>;
    removeAttribute(selectorOrElement: string | Element, name: string): Promise<void>;
    appendChild<T extends Node>(selectorOrElement: T | string, child: T): Promise<void>;
    remove(selectorOrElement: Element | string): Promise<void>;
    findElement(selector: string): Element;
    findElements(...selectors: string[]): Element[];
    get nextFrame(): Promise<any>;
}
export {};
