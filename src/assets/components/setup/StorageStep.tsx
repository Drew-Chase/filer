import {m} from "framer-motion";
import {useSetup} from "../../providers/SetupProvider.tsx";

export default function StorageStep()
{
    const {completeStep, gotoStep} = useSetup();
    return (
        <m.div
            key="storage"
            className={"h-full w-full bg-white/5 rounded-xl shadow-xl p-6 border border-white/20 overflow-y-scroll"}
            initial={{opacity: 0, x: -20}}
            animate={{opacity: 1, x: 0}}
            exit={{opacity: 0, x: 20}}
            transition={{duration: 0.25, ease: "easeInOut"}}
        >
            <div className={"w-full flex flex-col gap-4 items-center justify-center mt-8"}>
                <h1 className={"text-7xl font-bold"}>Storage Configuration</h1>
            </div>
        </m.div>
    );
}