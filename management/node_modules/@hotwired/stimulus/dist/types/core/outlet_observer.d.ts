import { Multimap } from "../multimap";
import { AttributeObserverDelegate } from "../mutation-observers";
import { SelectorObserverDelegate } from "../mutation-observers";
import { Context } from "./context";
import { Controller } from "./controller";
declare type OutletObserverDetails = {
    outletName: string;
};
export interface OutletObserverDelegate {
    outletConnected(outlet: Controller, element: Element, outletName: string): void;
    outletDisconnected(outlet: Controller, element: Element, outletName: string): void;
}
export declare class OutletObserver implements AttributeObserverDelegate, SelectorObserverDelegate {
    started: boolean;
    readonly context: Context;
    readonly delegate: OutletObserverDelegate;
    readonly outletsByName: Multimap<string, Controller>;
    readonly outletElementsByName: Multimap<string, Element>;
    private selectorObserverMap;
    private attributeObserverMap;
    constructor(context: Context, delegate: OutletObserverDelegate);
    start(): void;
    refresh(): void;
    stop(): void;
    stopSelectorObservers(): void;
    stopAttributeObservers(): void;
    selectorMatched(element: Element, _selector: string, { outletName }: OutletObserverDetails): void;
    selectorUnmatched(element: Element, _selector: string, { outletName }: OutletObserverDetails): void;
    selectorMatchElement(element: Element, { outletName }: OutletObserverDetails): boolean;
    elementMatchedAttribute(_element: Element, attributeName: string): void;
    elementAttributeValueChanged(_element: Element, attributeName: string): void;
    elementUnmatchedAttribute(_element: Element, attributeName: string): void;
    connectOutlet(outlet: Controller, element: Element, outletName: string): void;
    disconnectOutlet(outlet: Controller, element: Element, outletName: string): void;
    disconnectAllOutlets(): void;
    private updateSelectorObserverForOutlet;
    private setupSelectorObserverForOutlet;
    private setupAttributeObserverForOutlet;
    private selector;
    private attributeNameForOutletName;
    private getOutletNameFromOutletAttributeName;
    private get outletDependencies();
    private get outletDefinitions();
    private get dependentControllerIdentifiers();
    private get dependentContexts();
    private hasOutlet;
    private getOutlet;
    private getOutletFromMap;
    private get scope();
    private get schema();
    private get identifier();
    private get application();
    private get router();
}
export {};
