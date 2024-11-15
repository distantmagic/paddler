import { Controller } from "../../core/controller";
declare class BaseOutletController extends Controller {
    static outlets: string[];
    alphaOutlet: Controller | null;
    alphaOutlets: Controller[];
    alphaOutletElement: Element | null;
    alphaOutletElements: Element[];
    hasAlphaOutlet: boolean;
}
export declare class OutletController extends BaseOutletController {
    static classes: string[];
    static outlets: string[];
    static values: {
        alphaOutletConnectedCallCount: NumberConstructor;
        alphaOutletDisconnectedCallCount: NumberConstructor;
        betaOutletConnectedCallCount: NumberConstructor;
        betaOutletDisconnectedCallCount: NumberConstructor;
        betaOutletsInConnect: NumberConstructor;
        gammaOutletConnectedCallCount: NumberConstructor;
        gammaOutletDisconnectedCallCount: NumberConstructor;
        namespacedEpsilonOutletConnectedCallCount: NumberConstructor;
        namespacedEpsilonOutletDisconnectedCallCount: NumberConstructor;
    };
    betaOutlet: Controller | null;
    betaOutlets: Controller[];
    betaOutletElement: Element | null;
    betaOutletElements: Element[];
    hasBetaOutlet: boolean;
    namespacedEpsilonOutlet: Controller | null;
    namespacedEpsilonOutlets: Controller[];
    namespacedEpsilonOutletElement: Element | null;
    namespacedEpsilonOutletElements: Element[];
    hasNamespacedEpsilonOutlet: boolean;
    hasConnectedClass: boolean;
    hasDisconnectedClass: boolean;
    connectedClass: string;
    disconnectedClass: string;
    alphaOutletConnectedCallCountValue: number;
    alphaOutletDisconnectedCallCountValue: number;
    betaOutletConnectedCallCountValue: number;
    betaOutletDisconnectedCallCountValue: number;
    betaOutletsInConnectValue: number;
    gammaOutletConnectedCallCountValue: number;
    gammaOutletDisconnectedCallCountValue: number;
    namespacedEpsilonOutletConnectedCallCountValue: number;
    namespacedEpsilonOutletDisconnectedCallCountValue: number;
    connect(): void;
    alphaOutletConnected(_outlet: Controller, element: Element): void;
    alphaOutletDisconnected(_outlet: Controller, element: Element): void;
    betaOutletConnected(_outlet: Controller, element: Element): void;
    betaOutletDisconnected(_outlet: Controller, element: Element): void;
    gammaOutletConnected(_outlet: Controller, element: Element): void;
    namespacedEpsilonOutletConnected(_outlet: Controller, element: Element): void;
    namespacedEpsilonOutletDisconnected(_outlet: Controller, element: Element): void;
}
export {};
