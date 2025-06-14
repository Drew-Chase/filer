import SetupProgressStepper from "../components/setup/SetupProgressStepper.tsx";
import {cn} from "@heroui/react";
import {AnimatePresence, domAnimation, LazyMotion} from "framer-motion";
import {useSetup} from "../providers/SetupProvider.tsx";

export default function SetupPage()
{
    const {activeView} = useSetup();
    return (
        <div className={"w-[90vw] h-[100dvh] gap-4 m-auto flex items-center"}>
            <div className={
                cn(
                    "h-[80%] w-full flex-row flex gap-4"
                    // "bg-white/5 rounded-xl shadow-xl p-6 border border-white/20"
                )}>
                <SetupProgressStepper/>
                <LazyMotion features={domAnimation}>
                    <AnimatePresence mode={"wait"}>
                        {activeView}
                    </AnimatePresence>
                </LazyMotion>
            </div>
        </div>
    );
}

