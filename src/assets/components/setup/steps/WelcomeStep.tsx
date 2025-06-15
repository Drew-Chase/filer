import {m} from "framer-motion";
import {Button, Link} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import License from "../License.tsx";
import {useSetup} from "../../../providers/SetupProvider.tsx";

export default function WelcomeStep()
{
    const {completeStep, gotoStep, currentIndex} = useSetup();
    return (
        <m.div
            key="welcome"
            className={"h-full w-full bg-white/5 rounded-xl shadow-xl p-6 border border-white/20 overflow-y-scroll"}
            initial={{opacity: 0, x: -20}}
            animate={{opacity: 1, x: 0}}
            exit={{opacity: 0, x: 20}}
            transition={{duration: 0.25, ease: "easeInOut"}}
        >
            <div className={"w-full flex flex-col gap-4 items-center justify-center mt-8"}>
                <h1 className={"text-7xl font-bold"}>Welcome to Filer</h1>
                <p className={"text-lg text-center opacity-75"}>
                    Filer is a powerful remote file management tool designed to help you organize, search, and manage your files efficiently.
                </p>
                <div className={"w-[50vw] min-h-[300px] h-[calc(100dvh_-_550px)] grow overflow-y-auto p-4 bg-white/10 rounded-lg"}>
                    <License/>
                </div>
                <p className={"text-sm text-center opacity-75 italic"}>
                    * By clicking "Get Started", you agree to the terms of the license above.
                </p>
                <div className={"flex flex-row gap-4 w-[60%]"}>
                    <Button color={"secondary"} size={"lg"} variant={"solid"} radius={"full"} endContent={<Icon icon={"carbon:next-filled"}/>} className={"w-full"} onPress={() =>
                    {
                        completeStep(currentIndex);
                        gotoStep(currentIndex + 1);
                    }}>Get Started</Button>
                    <Button variant={"ghost"} size={"lg"} radius={"full"} className={"w-full"} startContent={<Icon icon="mdi:github"/>} as={Link} href={"https://github.com/drew-chase/filer/wiki"} target={"_blank"}>Wiki</Button>
                </div>
            </div>
        </m.div>
    );
}