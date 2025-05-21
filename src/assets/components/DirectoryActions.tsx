import {motion} from "framer-motion";
import {useState} from "react";
import {Icon} from "@iconify-icon/react";
import {cn} from "@heroui/react";

export function DirectoryActions()
{
    const [count, setCount] = useState(0);
    return (
        <motion.div
        >

            <DirectoryActionButton
                count={count}
                icon={"mage:file-download-fill"}
                tooltip={""}
                onPress={() =>
                {
                    setCount(count + 1);
                }}
            />

        </motion.div>
    );
}

type DirectoryActionButtonProps = {
    count: number;
    isPositiveCountRequired?: boolean;
    onPress: () => void;
    icon: string;
    tooltip: string;
    showCount?: boolean;
}

function DirectoryActionButton(props: DirectoryActionButtonProps)
{
    const [id] = useState(`directory-action-button-${Math.random().toString(36).substring(2, 15)}`);
    const [hovering, setHovering] = useState(false);
    const isPositiveCountRequired = props.isPositiveCountRequired ?? false;
    const showCount = props.showCount ?? true;
    if (props.count === 0 && isPositiveCountRequired) return null;
    return (
        <motion.div
            id={id}
            className={
                cn(
                    "flex items-center justify-center aspect-square w-10 h-10 rounded-full relative",
                    "bg-white text-black transition-colors duration-200",
                    "data-[hover=true]:bg-gray-300 data-[hover=true]:text-black cursor-pointer"
                )
            }
            initial={{opacity: 0, y: -10}}
            animate={{opacity: 1, y: 0}}
            transition={{duration: 0.2, delay: 0, type: "spring", ease: "easeInOut"}}
            onMouseEnter={() => setHovering(true)}
            onMouseLeave={() => setHovering(false)}
            onClick={props.onPress}
            data-hover={hovering}
        >
            {showCount && (
                <motion.div
                    key={props.count}
                    initial={{scale: 1, y: -5}}
                    animate={{scale: 1, y: 0}}
                    transition={{
                        type: "spring",
                        stiffness: 300,
                        damping: 10
                    }}
                    className={
                        cn(
                            "absolute top-0 right-0 -mt-2 -mr-2 z-20 p-1 aspect-square w-6 h-6",
                            "bg-white text-black text-xs rounded-full flex items-center justify-center"
                        )
                    }
                    data-hover={hovering}
                >
                    {props.count}
                </motion.div>
            )}
            <Icon icon={props.icon}/>
        </motion.div>
    );
}