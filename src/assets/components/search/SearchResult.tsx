import {Button, cn, Spinner, Tooltip} from "@heroui/react";
import {motion} from "framer-motion";
import FileEntryIcon from "../FileEntryIcon.tsx";
import {FilesystemEntry, FileSystem} from "../../ts/filesystem.ts";
import {Icon} from "@iconify-icon/react";
import {useEffect, useState, useRef} from "react";
import {useNavigate} from "react-router-dom"; // Add useRef import

type SearchResultProps = {
    query: string;
    onFocus: () => void;
    onBlur: () => void;
    focused: boolean;
    onNavigate: () => void;
}

export default function SearchResult(props: SearchResultProps)
{
    const [isLoading, setIsLoading] = useState(false);
    const [items, setItems] = useState<FilesystemEntry[]>([]);

    const abortControllerRef = useRef<AbortController>(new AbortController());

    useEffect(() =>
    {
        setIsLoading(true);

        abortControllerRef.current.abort();
        abortControllerRef.current = new AbortController();

        FileSystem
            .search(props.query, false, abortControllerRef.current.signal)
            .then(setItems)
            .catch(error =>
            {
                if (error.name !== "AbortError")
                {
                    console.error("Search error:", error);
                }
            })
            .finally(() => setIsLoading(false));

        return () =>
        {
            abortControllerRef.current.abort();
        };
    }, [props.query]);

    return (
        <motion.div
            tabIndex={0}
            onFocus={props.onFocus}
            onBlur={props.onBlur}
            initial={{opacity: 0, y: -20, height: 0}}
            animate={{opacity: 1, y: 0, height: 200}}
            exit={{opacity: 0, y: -20, height: 0}}
            transition={{duration: .5, delay: 0, type: "spring", ease: "easeInOut"}}
            className={
                cn(
                    "absolute top-16 w-1/2 -ml-4 max-h-[500px] rounded-lg",
                    "bg-[#2f115c]/50 border-primary/50 border-1 backdrop-brightness-[.5] backdrop-saturate-[1.5] backdrop-blur-lg",
                    "flex flex-col overflow-y-auto",
                    "data-[focused=true]:border-primary data-[focused=true]:border-1"
                )
            }
            data-focused={props.focused}
        >
            {isLoading ? (<Spinner size={"lg"} className={"mt-4"}/>) : (
                <>
                    {items.length === 0 ? (<p className={"mx-auto mt-4 italic opacity-70"}>No results found!</p>) : (<>
                        {items.map((entry) => (
                            <SearchResultItem
                                entry={entry}
                                onNavigate={props.onNavigate}
                            />
                        ))}
                    </>)}
                </>
            )}
        </motion.div>
    );
}

function SearchResultItem({entry, onNavigate}: { entry: FilesystemEntry, onNavigate: () => void })
{
    const navigate = useNavigate();
    return (
        <div className="flex flex-row items-center gap-2 hover:bg-white/10 p-2">
            <FileEntryIcon entry={entry}/>
            <div className={"flex flex-col w-full"}>
                <p>{entry.filename}</p>
                <p className={"text-sm opacity-50 italic"}>{entry.path}</p>
            </div>
            <div className={"flex flex-row items-center gap-2 mr-4"}>
                <Tooltip content={"Open Parent Directory"} className={"pointer-events-none"}>
                    <Button
                        variant={"light"}
                        className={"w-10 h-10 min-w-10 min-h-10"}
                        onPress={() =>
                        {
                            navigate(`/files/${entry.path.replace(/\\/g, "/").split("/").slice(0, -1).join("/")}`);
                            onNavigate();
                        }}
                    >
                        <Icon
                            icon={"mage:folder-fill"}
                            className={"data-[directory=true]:text-blue-500"}
                            aria-hidden="true"
                        />
                    </Button>
                </Tooltip>
            </div>
        </div>
    );
}