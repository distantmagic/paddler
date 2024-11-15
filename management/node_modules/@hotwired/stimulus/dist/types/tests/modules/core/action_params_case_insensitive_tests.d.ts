import ActionParamsTests from "./action_params_tests";
export default class ActionParamsCaseInsensitiveTests extends ActionParamsTests {
    identifier: string[];
    fixtureHTML: string;
    expectedParamsForCamelCase: {
        id: number;
        multiWordExample: string;
        payload: {
            value: number;
        };
        active: boolean;
        empty: string;
        inactive: boolean;
    };
    "test clicking on the element does return its params"(): Promise<void>;
    "test global event return element params where the action is defined"(): Promise<void>;
    "test passing params to namespaced controller"(): Promise<void>;
}
