import { ActionDescriptor } from "./action_descriptor";
import { Token } from "../mutation-observers";
import { Schema } from "./schema";
export declare class Action {
    readonly element: Element;
    readonly index: number;
    readonly eventTarget: EventTarget;
    readonly eventName: string;
    readonly eventOptions: AddEventListenerOptions;
    readonly identifier: string;
    readonly methodName: string;
    readonly keyFilter: string;
    readonly schema: Schema;
    static forToken(token: Token, schema: Schema): Action;
    constructor(element: Element, index: number, descriptor: Partial<ActionDescriptor>, schema: Schema);
    toString(): string;
    shouldIgnoreKeyboardEvent(event: KeyboardEvent): boolean;
    shouldIgnoreMouseEvent(event: MouseEvent): boolean;
    get params(): {
        [key: string]: any;
    };
    private get eventTargetName();
    private get keyMappings();
    private keyFilterDissatisfied;
}
export declare function getDefaultEventNameForElement(element: Element): string | undefined;
