import {Input, Kbd} from "@heroui/react";
import {useState} from "react";
import SearchResult from "./SearchResult.tsx";
import {AnimatePresence, motion} from "framer-motion";

export default function SearchInput()
{
    const [isFocused, setIsFocused] = useState(false);
    const [isResultFocused, setIsResultFocused] = useState(false);
    const [search, setSearch] = useState("");
    return (
        <div className={"z-20 w-full"}>
            <motion.div
                className={"fixed top-0 bottom-0 left-0 right-0 h-dvh backdrop-blur-sm backdrop-contrast-[1.1] pointer-events-none"}
                initial={{opacity: 0}}
                animate={{opacity: (isFocused || isResultFocused) && search !== "" ? 1 : 0}}
                transition={{duration: 1.2, delay: 0, type: "spring", ease: "easeInOut"}}
            >

            </motion.div>
            <Input
                label={"Search"}
                placeholder={"Search for files or folders..."}
                size={"sm"}
                className={"w-full"}
                classNames={{
                    inputWrapper: "bg-white/20 data-[hover]:bg-white/15 group-data-[focus]:bg-white/10 group-data-[focus]:border-primary border-1 border-transparent"
                }}
                endContent={<Kbd keys={["command"]}>K</Kbd>}
                onFocus={() => setIsFocused(true)}
                onBlur={() => setIsFocused(false)}
                onValueChange={setSearch}
                value={search}
            />
            <AnimatePresence>
                {(isFocused || isResultFocused) && search !== "" &&
                    <SearchResult
                        query={search}
                        onFocus={() => setIsResultFocused(true)}
                        onBlur={() => setIsResultFocused(false)}
                        focused={isResultFocused}
                        onNavigate={() =>
                        {
                            setIsFocused(false);
                            setIsResultFocused(false);
                        }}
                    />
                }
            </AnimatePresence>
        </div>
    );
}