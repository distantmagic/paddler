import { Constructor } from "./constructor";
export declare function ValuePropertiesBlessing<T>(constructor: Constructor<T>): PropertyDescriptorMap;
export declare function propertiesForValueDefinitionPair<T>(valueDefinitionPair: ValueDefinitionPair, controller?: string): PropertyDescriptorMap;
export declare type ValueDescriptor = {
    type: ValueType;
    key: string;
    name: string;
    defaultValue: ValueTypeDefault;
    hasCustomDefaultValue: boolean;
    reader: Reader;
    writer: Writer;
};
export declare type ValueDescriptorMap = {
    [attributeName: string]: ValueDescriptor;
};
export declare type ValueDefinitionMap = {
    [token: string]: ValueTypeDefinition;
};
export declare type ValueDefinitionPair = [string, ValueTypeDefinition];
export declare type ValueTypeConstant = typeof Array | typeof Boolean | typeof Number | typeof Object | typeof String;
export declare type ValueTypeDefault = Array<any> | boolean | number | Object | string;
export declare type ValueTypeObject = Partial<{
    type: ValueTypeConstant;
    default: ValueTypeDefault;
}>;
export declare type ValueTypeDefinition = ValueTypeConstant | ValueTypeDefault | ValueTypeObject;
export declare type ValueType = "array" | "boolean" | "number" | "object" | "string";
export declare function parseValueTypeConstant(constant?: ValueTypeConstant): "object" | "string" | "number" | "array" | "boolean" | undefined;
export declare function parseValueTypeDefault(defaultValue?: ValueTypeDefault): "object" | "string" | "number" | "array" | "boolean" | undefined;
declare type ValueTypeObjectPayload = {
    controller?: string;
    token: string;
    typeObject: ValueTypeObject;
};
export declare function parseValueTypeObject(payload: ValueTypeObjectPayload): "object" | "string" | "number" | "array" | "boolean" | undefined;
declare type ValueTypeDefinitionPayload = {
    controller?: string;
    token: string;
    typeDefinition: ValueTypeDefinition;
};
export declare function parseValueTypeDefinition(payload: ValueTypeDefinitionPayload): ValueType;
export declare function defaultValueForDefinition(typeDefinition: ValueTypeDefinition): ValueTypeDefault;
declare type Reader = (value: string) => any;
declare type Writer = (value: any) => string;
export {};
