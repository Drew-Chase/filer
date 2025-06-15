import {createContext, JSX, ReactNode, useCallback, useContext, useState} from "react";
import WelcomeStep from "../components/setup/steps/WelcomeStep.tsx";
import StorageStep from "../components/setup/steps/StorageStep.tsx";
import NetworkStep from "../components/setup/steps/NetworkStep.tsx";
import AccountStep from "../components/setup/steps/AccountStep.tsx";
import FinishStep from "../components/setup/steps/FinishStep.tsx";

export type Step = {
    title: string;
    description: string;
    completed: boolean;
    active: boolean;
    view: JSX.Element | null;
    available: boolean;
}

interface SetupContextType
{
    steps: Step[];
    gotoStep: (index: number) => void;
    completeStep: (index: number) => void;
    activeView: JSX.Element | null;
    currentIndex: number;
}

const createDefaultSteps = (): Step[] => [
    {
        title: "Welcome to Filer",
        description: "Let's get started with the setup process.",
        completed: false,
        active: true,
        available: true,
        view: <WelcomeStep/>
    },
    {
        title: "Configure your storage",
        description: "Choose where you want to store your files.",
        completed: false,
        active: false,
        available: false,
        view: <StorageStep/>
    },
    {
        title: "Configure your network",
        description: "Set up your network settings.",
        completed: false,
        active: false,
        available: false,
        view: <NetworkStep/>
    },
    {
        title: "Set up your account",
        description: "Create an account to manage your files.",
        completed: false,
        active: false,
        available: false,
        view: <AccountStep/>
    },
    {
        title: "Complete setup",
        description: "Finish the setup process and start using Filer.",
        completed: false,
        active: false,
        available: false,
        view: <FinishStep/>
    }
];

const SetupContext = createContext<SetupContextType | undefined>(undefined);

export function SetupProvider({children}: { children: ReactNode })
{
    const [steps, SetSteps] = useState<Step[]>(() => createDefaultSteps());

    const gotoStep = useCallback((index: number) =>
    {
        SetSteps((prevSteps) => prevSteps.map((step, i) => ({
            ...step,
            active: i === index,
            available: i === index || step.available
        })));
    }, []);

    const completeStep = useCallback((index: number) =>
    {
        SetSteps((prevSteps) => prevSteps.map((step, i) => ({
            ...step,
            completed: i === index ? true : step.completed,
            active: i === index + 1 ? true : step.active
        })));
    }, []);

    return (
        <SetupContext.Provider value={{steps, gotoStep, completeStep, activeView: steps.find(step => step.active)?.view || null, currentIndex: steps.findIndex(step => step.active)}}>
            {children}
        </SetupContext.Provider>
    );
}

export function useSetup(): SetupContextType
{
    let context = useContext(SetupContext);
    if (!context)
    {
        throw new Error("useSetup must be used within a SetupProvider");
    }
    return context;
}

// Add this at the bottom to handle HMR properly
if (import.meta.hot)
{
    import.meta.hot.accept();
}