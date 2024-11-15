import { OutletController } from "../../controllers/outlet_controller";
declare class OutletOrderController extends OutletController {
    connect(): void;
}
declare const OutletOrderTests_base: import("../../../core/constructor").Constructor<import("../../cases/controller_test_case").ControllerTests<OutletOrderController>>;
export default class OutletOrderTests extends OutletOrderTests_base {
    fixtureHTML: string;
    get identifiers(): string[];
    "test can access outlets in connect() even if they are referenced before they are connected"(): Promise<void>;
}
export {};
