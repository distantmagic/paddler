import type { Controller } from "./controller";
export declare type ActionDescriptorFilters = Record<string, ActionDescriptorFilter>;
export declare type ActionDescriptorFilter = (options: ActionDescriptorFilterOptions) => boolean;
declare type ActionDescriptorFilterOptions = {
    name: string;
    value: boolean;
    event: Event;
    element: Element;
    controller: Controller<Element>;
};
export declare const defaultActionDescriptorFilters: ActionDescriptorFilters;
export interface ActionDescriptor {
    eventTarget: EventTarget;
    eventOptions: AddEventListenerOptions;
    eventName: string;
    identifier: string;
    methodName: string;
    keyFilter: string;
}
export declare function parseActionDescriptorString(descriptorString: string): Partial<ActionDescriptor>;
export declare function stringifyEventTarget(eventTarget: EventTarget): "window" | "document" | undefined;
export {};
