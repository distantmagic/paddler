import { ValueController } from "../../controllers/value_controller";
declare const ValuePropertiesTests_base: import("../../../core/constructor").Constructor<import("../../cases/controller_test_case").ControllerTests<ValueController>>;
export default class ValuePropertiesTests extends ValuePropertiesTests_base {
    "test parseValueTypeConstant"(): void;
    "test parseValueTypeDefault"(): void;
    "test parseValueTypeObject"(): void;
    "test parseValueTypeDefinition booleans"(): void;
    "test defaultValueForDefinition"(): void;
}
export {};
