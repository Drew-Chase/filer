import {m} from "framer-motion";
import {Button, Link} from "@heroui/react";
import {Icon} from "@iconify-icon/react";

export default function FinishStep()
{
    return (
        <m.div
            key="finish"
            className={"h-full w-full bg-white/5 rounded-xl shadow-xl p-6 border border-white/20 overflow-y-scroll relative"}
            initial={{opacity: 0, x: -20}}
            animate={{opacity: 1, x: 0}}
            exit={{opacity: 0, x: 20}}
            transition={{duration: 0.25, ease: "easeInOut"}}
        >
            <Button
                variant={"light"}
                size={"lg"}
                radius={"full"}
                className={"w-12 h-12 absolute top-4 right-4 aspect-square text-large p-0 min-w-0 min-h-0"}
                as={Link}
                href={"https://github.com/Drew-Chase/filer/wiki/Setup#configure-your-storage"}
                target={"_blank"}
            >
                <Icon icon={"mingcute:question-fill"}/>
            </Button>
            <div className={"w-full flex flex-col gap-4 items-center justify-center mt-8"}>
                <h1 className={"text-7xl font-bold"}>Almost Done!</h1>


                <Button
                    color={"secondary"}
                    size={"lg"}
                    variant={"solid"}
                    radius={"full"}
                    className={"w-12 h-12 absolute bottom-4 right-4 aspect-square p-0 min-w-0 min-h-0"}
                    onPress={() =>
                    {
                    }}
                >
                    <Icon icon={"maki:arrow"}/>
                </Button>
            </div>
        </m.div>
    );
}