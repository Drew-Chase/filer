import logo from "../../images/filer-logo.svg";
import {cn, Image, Progress} from "@heroui/react";
import {domAnimation, LazyMotion, m} from "framer-motion";
import {ComponentProps} from "react";
import {Step, useSetup} from "../../providers/SetupProvider.tsx";


export default function SetupProgressStepper()
{
    const {steps, gotoStep} = useSetup();

    return (
        <div className={"h-full min-w-[300px] w-[300px] bg-white/5 rounded-xl shadow-xl p-6 border border-white/20 flex flex-col gap-4 items-center"}>
            <div className={"flex flex-row items-center gap-2 text-xl font-bold"}><Image src={logo} width={32}/> Filer</div>
            <Progress value={steps.filter(i => i.completed).length} minValue={0} maxValue={steps.length} color={"primary"} size={"sm"}/>
            <div className={"flex flex-col gap-6 overflow-y-scroll h-full"}>
                {steps.map((step, index) => (
                    <SetupStep
                        key={`step-${index}`}
                        step={step}
                        index={index}
                        isLastStep={index === steps.length - 1}
                        setActive={() => gotoStep(index)}
                    />
                ))}

            </div>
        </div>
    );
}


function SetupStep({step, index, isLastStep, setActive}: { step: Step, index: number, isLastStep: boolean, setActive: () => void; })
{
    return (
        <LazyMotion features={domAnimation}>
            <div className={
                cn(
                    "flex flex-row items-center gap-4 data-[available=true]:cursor-pointer data-[available=false]:cursor-default relative h-24 shrink-0",
                    "data-[completed=false]:opacity-50 hover:data-[available=true]:data-[completed=false]:opacity-75 transition-opacity duration-200",
                    "data-[active=true]:!opacity-100"
                )
            }
                 data-completed={step.completed}
                 data-active={step.active}
                 data-available={step.available}
                 onClick={step.available ? setActive : undefined}
            >
                <div
                    className={
                        cn(
                            "w-10 h-10 rounded-full flex items-center justify-center",
                            "bg-transparent border-white border-2 font-bold aspect-square",
                            "data-[active=true]:text-primary data-[active=true]:border-primary",
                            "data-[completed=true]:text-white data-[completed=true]:bg-primary data-[completed=true]:border-primary"
                        )
                    }
                    data-completed={step.completed}
                    data-active={step.active}
                >
                    {step.completed ? (<CheckIcon width={24}/>) : (index + 1)}
                    {!isLastStep && (
                        <div
                            className={"absolute w-[2px] h-10 bg-white/30 rounded-full -bottom-6 data-[completed=true]:bg-primary transition-colors duration-200"}
                        >
                            <m.div className={"bg-primary"}
                                   initial={{height: 0}}
                                   animate={{height: step.completed ? "100%" : 0}}
                                   transition={{
                                       type: "tween",
                                       ease: "easeOut",
                                       duration: 0.3,
                                       delay: 0.1
                                   }}
                            />
                        </div>
                    )}
                </div>
                <div className={"flex flex-col gap-2"}>
                    <div className={"text-lg font-semibold data-[active=true]:text-primary"} data-completed={step.completed} data-active={step.active}>{step.title}</div>
                    <m.div
                        className={"text-sm text-gray-400"}
                        initial={{opacity: 0, height: 0}}
                        animate={{opacity: step.active ? 1 : 0, height: step.active ? "auto" : 0}}
                    >
                        {step.description}
                    </m.div>
                </div>
            </div>
        </LazyMotion>
    );
}

function CheckIcon(props: ComponentProps<"svg">)
{
    return (
        <svg {...props} fill="none" stroke="currentColor" strokeWidth={2} viewBox="0 0 24 24">
            <m.path
                animate={{pathLength: 1}}
                d="M5 13l4 4L19 7"
                initial={{pathLength: 0}}
                strokeLinecap="round"
                strokeLinejoin="round"
                transition={{
                    delay: 0.2,
                    type: "tween",
                    ease: "easeOut",
                    duration: 0.3
                }}
            />
        </svg>
    );
}