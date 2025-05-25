import {motion} from "framer-motion";
import {Input} from "@heroui/react";
import {useEffect, useState} from "react";
import {useFileSystemEntry} from "../providers/FileSystemEntryProvider.tsx";
import {useLocation} from "react-router-dom";

export default function TableFind()
{
    const [isVisible, setIsVisible] = useState(false);
    const {currentDirectoryFilter, onCurrentDirectoryFilterChange} = useFileSystemEntry();
    const {pathname} = useLocation();
    useEffect(() =>
    {
        onCurrentDirectoryFilterChange("");
    }, [pathname]);
    useEffect(() =>
    {
        const handleKeyUp = (e: KeyboardEvent) =>
        {
            if (e.ctrlKey && e.key === "f")
            {
                e.preventDefault();
                e.stopPropagation();

                setIsVisible(prev =>
                {
                    const value = !prev;
                    if (value)
                    {
                        const input = document.getElementById("table-find-input") as HTMLInputElement;
                        if (input)
                        {
                            input.focus();
                            input.addEventListener("blur", () => setIsVisible(false), {once: true});
                        }
                    } else
                    {
                        onCurrentDirectoryFilterChange("");
                    }
                    return value;
                });
            }
        };

        window.addEventListener("keydown", handleKeyUp, {capture: true});


        return () =>
        {
            window.removeEventListener("keydown", handleKeyUp, {capture: true});
        };
    }, []);
    return (
        <motion.div
            className={"absolute right-12 bottom-12 z-10 flex items-center justify-center gap-2 w-[300px] data-[visible=false]:pointer-events-none"}
            initial={{opacity: 0, y: 20}}
            animate={{opacity: isVisible ? 1 : 0, y: isVisible ? 0 : 20}}
            transition={{duration: 0.2, delay: 0, type: "spring", ease: "easeInOut"}}
            data-visible={isVisible}
        >
            <Input
                id={"table-find-input"}
                size={"sm"}
                placeholder={"Find in directory..."}
                classNames={{
                    inputWrapper: "bg-white/20 data-[hover]:bg-white/15 group-data-[focus]:bg-white/10 group-data-[focus]:border-primary border-1 border-transparent backdrop-blur-lg backdrop-brightness-75"
                }}
                value={currentDirectoryFilter}
                onValueChange={onCurrentDirectoryFilterChange}
                onKeyDown={e =>
                {
                    if (e.key === "Escape" || e.key === "Enter")
                    {
                        (e.target as HTMLInputElement).blur();
                        setIsVisible(false);
                        e.preventDefault();
                    }
                }}
                data-visible={isVisible}
            />
        </motion.div>
    );
}