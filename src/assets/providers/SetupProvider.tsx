import {createContext, JSX, ReactNode, useCallback, useContext, useState} from "react";
import WelcomeStep from "../components/setup/WelcomeStep.tsx";

export type Step = {
    title: string;
    description: string;
    completed: boolean;
    active: boolean;
    view: JSX.Element | null;
}

interface SetupContextType
{
    steps: Step[];
    gotoStep: (index: number) => void;
    completeStep: (index: number) => void;
    activeView: JSX.Element | null;
}

const DEFAULT_STEPS: Step[] = [
    {
        title: "Welcome to Filer",
        description: "Let's get started with the setup process.",
        completed: false,
        active: true,
        view: <WelcomeStep/>
    },
    {
        title: "Configure your storage",
        description: "Choose where you want to store your files.",
        completed: false,
        active: false,
        view: null
    },
    {
        title: "Set up your account",
        description: "Create an account to manage your files.",
        completed: false,
        active: false,
        view: null
    },
    {
        title: "Complete setup",
        description: "Finish the setup process and start using Filer.",
        completed: false,
        active: false,
        view: null
    }
];

const SetupContext = createContext<SetupContextType | undefined>(undefined);

export function SetupProvider({children}: { children: ReactNode })
{
    const [steps, SetSteps] = useState<Step[]>(DEFAULT_STEPS);

    const gotoStep = useCallback((index: number) =>
    {
        SetSteps((prevSteps) => prevSteps.map((step, i) => ({
            ...step,
            active: i === index
        })));
    }, [steps]);
    const completeStep = useCallback((index: number) =>
    {
        SetSteps((prevSteps) => prevSteps.map((step, i) => ({
            ...step,
            completed: i === index ? true : step.completed,
            active: i === index + 1 ? true : step.active
        })));
    }, [steps]);

    return (
        <SetupContext.Provider value={{steps, gotoStep, completeStep, activeView: steps.find(step => step.active)?.view || null}}>
            {children}
        </SetupContext.Provider>
    );
}

export function useSetup(): SetupContextType
{
    const context = useContext(SetupContext);
    if (!context)
    {
        throw new Error("useSetup must be used within a SetupProvider");
    }
    return context;
}