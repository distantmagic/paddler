export interface Schema {
    controllerAttribute: string;
    actionAttribute: string;
    targetAttribute: string;
    targetAttributeForScope(identifier: string): string;
    outletAttributeForScope(identifier: string, outlet: string): string;
    keyMappings: {
        [key: string]: string;
    };
}
export declare const defaultSchema: Schema;
