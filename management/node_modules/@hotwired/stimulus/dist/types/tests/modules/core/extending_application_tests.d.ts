import { Application } from "../../../core/application";
import { DOMTestCase } from "../../cases/dom_test_case";
export default class ExtendingApplicationTests extends DOMTestCase {
    application: Application;
    runTest(testName: string): Promise<void>;
    setup(): Promise<void>;
    teardown(): Promise<void>;
    "test extended class method is supported when using MyApplication.start()"(): Promise<void>;
}
