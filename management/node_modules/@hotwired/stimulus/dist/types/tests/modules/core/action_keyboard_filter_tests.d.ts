import { LogControllerTestCase } from "../../cases/log_controller_test_case";
import { Schema } from "../../../core/schema";
import { Application } from "../../../core/application";
export default class ActionKeyboardFilterTests extends LogControllerTestCase {
    schema: Schema;
    application: Application;
    identifier: string[];
    fixtureHTML: string;
    "test ignore event handlers associated with modifiers other than Enter"(): Promise<void>;
    "test ignore event handlers associated with modifiers other than Space"(): Promise<void>;
    "test ignore event handlers associated with modifiers other than Tab"(): Promise<void>;
    "test ignore event handlers associated with modifiers other than Escape"(): Promise<void>;
    "test ignore event handlers associated with modifiers other than ArrowUp"(): Promise<void>;
    "test ignore event handlers associated with modifiers other than ArrowDown"(): Promise<void>;
    "test ignore event handlers associated with modifiers other than ArrowLeft"(): Promise<void>;
    "test ignore event handlers associated with modifiers other than ArrowRight"(): Promise<void>;
    "test ignore event handlers associated with modifiers other than Home"(): Promise<void>;
    "test ignore event handlers associated with modifiers other than End"(): Promise<void>;
    "test keyup"(): Promise<void>;
    "test global event"(): Promise<void>;
    "test custom keymapping: a"(): Promise<void>;
    "test custom keymapping: b"(): Promise<void>;
    "test custom keymapping: unknown c"(): Promise<void>;
    "test ignore event handlers associated with modifiers other than shift+a"(): Promise<void>;
    "test ignore event handlers associated with modifiers other than a"(): Promise<void>;
    "test ignore event handlers associated with modifiers other than ctrol+shift+a"(): Promise<void>;
    "test ignore filter syntax when not a keyboard event"(): Promise<void>;
    "test ignore filter syntax when not a keyboard event (case2)"(): Promise<void>;
}
