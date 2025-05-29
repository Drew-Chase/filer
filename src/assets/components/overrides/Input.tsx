import {Input as HeroInput, InputProps} from "@heroui/react";

export default function Input(props: InputProps)
{
    return (
        <HeroInput
            {...props}
            classNames={{
                inputWrapper: "bg-white/5 data-[hover]:bg-white/10 group-data-[focus]:bg-white/5 data-[focus]:outline-primary outline-2 outline-transparent"
            }}
        />
    );
}