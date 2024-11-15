import { ElementObserverDelegate } from "./element_observer";
export interface SelectorObserverDelegate {
    selectorMatched(element: Element, selector: string, details: object): void;
    selectorUnmatched(element: Element, selector: string, details: object): void;
    selectorMatchElement?(element: Element, details: object): boolean;
}
export declare class SelectorObserver implements ElementObserverDelegate {
    private readonly elementObserver;
    private readonly delegate;
    private readonly matchesByElement;
    private readonly details;
    _selector: string | null;
    constructor(element: Element, selector: string, delegate: SelectorObserverDelegate, details: object);
    get started(): boolean;
    get selector(): string | null;
    set selector(selector: string | null);
    start(): void;
    pause(callback: () => void): void;
    stop(): void;
    refresh(): void;
    get element(): Element;
    matchElement(element: Element): boolean;
    matchElementsInTree(tree: Element): Element[];
    elementMatched(element: Element): void;
    elementUnmatched(element: Element): void;
    elementAttributeChanged(element: Element, _attributeName: string): void;
    private selectorMatched;
    private selectorUnmatched;
}
