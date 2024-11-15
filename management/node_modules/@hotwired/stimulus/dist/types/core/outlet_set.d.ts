import { Scope } from "./scope";
export declare class OutletSet {
    readonly scope: Scope;
    readonly controllerElement: Element;
    constructor(scope: Scope, controllerElement: Element);
    get element(): Element;
    get identifier(): string;
    get schema(): import("./schema").Schema;
    has(outletName: string): boolean;
    find(...outletNames: string[]): Element | undefined;
    findAll(...outletNames: string[]): Element[];
    getSelectorForOutletName(outletName: string): string | null;
    private findOutlet;
    private findAllOutlets;
    private findElement;
    private findAllElements;
    private matchesElement;
}
