import { Application } from "../../core/application";
import { DOMTestCase } from "./dom_test_case";
import { Schema } from "../../core/schema";
export declare class TestApplication extends Application {
    handleError(error: Error, _message: string, _detail: object): void;
}
export declare class ApplicationTestCase extends DOMTestCase {
    schema: Schema;
    application: Application;
    runTest(testName: string): Promise<void>;
    setupApplication(): void;
}
