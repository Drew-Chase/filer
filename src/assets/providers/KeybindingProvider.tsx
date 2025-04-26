import {createContext, Dispatch, ReactNode, SetStateAction, useContext, useState} from "react";

interface KeybindingContextType
{
    keybinding: string | null;
    setKeybinding: Dispatch<SetStateAction<string | null>>;
}

const KeybindingContext = createContext<KeybindingContextType | undefined>(undefined);

export function KeybindingProvider({children}: { children: ReactNode })
{
    const [keybinding, setKeybinding] = useState<string | null>(null);

    return (
        <KeybindingContext.Provider value={{keybinding, setKeybinding}}>
            {children}
        </KeybindingContext.Provider>
    );
}

export function useKeybinding(): KeybindingContextType
{
    const context = useContext(KeybindingContext);
    if (!context)
    {
        throw new Error("useKeybinding must be used within a KeybindingProvider");
    }
    return context;
}