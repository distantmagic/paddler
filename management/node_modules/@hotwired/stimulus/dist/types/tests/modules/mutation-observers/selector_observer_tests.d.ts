import { SelectorObserver, SelectorObserverDelegate } from "../../../mutation-observers/selector_observer";
import { ObserverTestCase } from "../../cases/observer_test_case";
export default class SelectorObserverTests extends ObserverTestCase implements SelectorObserverDelegate {
    attributeName: string;
    selector: string;
    details: {
        some: string;
    };
    fixtureHTML: string;
    observer: SelectorObserver;
    "test should match when observer starts"(): Promise<void>;
    "test should match when element gets appended"(): Promise<void>;
    "test should not match/unmatch when the attribute gets updated and matching selector persists"(): Promise<void>;
    "test should match when attribute gets updated and start to matche selector"(): Promise<void>;
    "test should unmatch when attribute gets updated but matching attribute value gets removed"(): Promise<void>;
    "test should unmatch when attribute gets removed"(): Promise<void>;
    "test should unmatch when element gets removed"(): Promise<void>;
    "test should not match/unmatch when observer is paused"(): Promise<void>;
    get element(): Element;
    get div1(): Element;
    get div2(): Element;
    selectorMatched(element: Element, selector: string, details: object): void;
    selectorUnmatched(element: Element, selector: string, details: object): void;
}
