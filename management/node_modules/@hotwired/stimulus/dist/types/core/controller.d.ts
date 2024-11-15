import { Application } from "./application";
import { Constructor } from "./constructor";
import { Context } from "./context";
import { OutletPropertiesBlessing } from "./outlet_properties";
import { ValueDefinitionMap } from "./value_properties";
export declare type ControllerConstructor = Constructor<Controller>;
declare type DispatchOptions = Partial<{
    target: Element | Window | Document;
    detail: Object;
    prefix: string;
    bubbles: boolean;
    cancelable: boolean;
}>;
export declare class Controller<ElementType extends Element = Element> {
    static blessings: (typeof OutletPropertiesBlessing)[];
    static targets: string[];
    static outlets: string[];
    static values: ValueDefinitionMap;
    static get shouldLoad(): boolean;
    static afterLoad(_identifier: string, _application: Application): void;
    readonly context: Context;
    constructor(context: Context);
    get application(): Application;
    get scope(): import("./scope").Scope;
    get element(): ElementType;
    get identifier(): string;
    get targets(): import("./target_set").TargetSet;
    get outlets(): import("./outlet_set").OutletSet;
    get classes(): import("./class_map").ClassMap;
    get data(): import("./data_map").DataMap;
    initialize(): void;
    connect(): void;
    disconnect(): void;
    dispatch(eventName: string, { target, detail, prefix, bubbles, cancelable, }?: DispatchOptions): CustomEvent<Object>;
}
export {};
