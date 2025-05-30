import {FavoriteItem, useFavorites} from "../../providers/FavoritesProvider.tsx";
import {useFileSystemEntry} from "../../providers/FileSystemEntryProvider.tsx";
import {useState} from "react";
import {Icon} from "@iconify-icon/react";
import {motion} from "framer-motion";
import {Button, Tooltip} from "@heroui/react";
import {useSortable} from "@dnd-kit/sortable";
import {CSS} from "@dnd-kit/utilities";

export function FavoriteEntry(props: { entry: FavoriteItem })
{
    const {removeFavorite} = useFavorites();
    const {navigate} = useFileSystemEntry();
    const [isHovering, setIsHovering] = useState(false);
    const [isRemoving, setIsRemoving] = useState(false);
    const {
        attributes,
        listeners,
        setNodeRef,
        transform,
        transition,
        isDragging
    } = useSortable({
        id: props.entry.path
    });


    // Generate dnd-kit transform CSS string
    const style = {
        transform: CSS.Transform.toString(transform),
        transition
    };

    return (
        <div
            ref={setNodeRef}
            style={style}
            className={`flex flex-row ${isDragging ? "z-10" : ""}`}
        >
            <motion.div
                initial={{opacity: 0, x: 0}}
                animate={{
                    opacity: isRemoving ? 0 : 1,
                    x: isRemoving ? -40 : 0,
                    scale: isDragging ? 1.02 : 1,
                    transition: {
                        opacity: {duration: 1, delay: 0, type: "spring", ease: "easeInOut"},
                        x: {duration: 1, delay: 0, type: "spring", ease: "easeInOut"},
                        scale: {duration: 0.3, type: "spring"}
                    }
                }}
                className={"flex flex-row w-full"}
                onMouseOver={() => setIsHovering(true)}
                onMouseLeave={() => setIsHovering(false)}
            >
                <div className={"flex flex-row w-full items-center p-2 gap-2 hover:bg-white/20 transition-colors duration-200 rounded-lg"}>
                    <Icon
                        {...attributes}
                        {...listeners}
                        icon={"mage:folder-fill"}
                        className={"text-xl text-blue-500 cursor-grab"}
                        aria-hidden="true"
                    />
                    <div className={"flex flex-col cursor-pointer"} onClick={() => navigate(props.entry.path)}>
                        <p className={"text-tiny max-w-[270px] truncate"}>{props.entry.name}</p>
                        <motion.div
                            initial={{opacity: 0, height: 0}}
                            animate={{opacity: isHovering ? 1 : 0, height: isHovering ? 16 : 0}}
                            transition={{duration: 0.5, delay: 0, type: "spring", ease: "easeInOut"}}
                        >
                            <Tooltip content={decodeURIComponent(props.entry.path)} delay={1500} closeDelay={0} className={"pointer-events-none"}>
                                <p className={"text-tiny italic font-light truncate opacity-50 pr-2 max-w-[270px]"}>{decodeURIComponent(props.entry.path)}</p>
                            </Tooltip>
                        </motion.div>
                    </div>
                </div>
                <motion.div
                    initial={{opacity: 0, width: 0}}
                    animate={{opacity: isHovering ? 1 : 0, width: isHovering ? 48 : 0}}
                    className={"flex flex-row items-center gap-2 overflow-hidden"}
                >
                    <Button variant={"light"} className={"min-w-0 min-h-0 w-10 h-10"} onPress={() =>
                    {
                        setIsRemoving(true);
                        setTimeout(() =>
                        {
                            removeFavorite(props.entry.path);
                            setIsRemoving(false);
                        }, 1000);
                    }}>
                        <Icon
                            icon={"mage:trash-fill"}
                            className={"text-danger"}
                            aria-hidden="true"
                        />
                    </Button>
                </motion.div>
            </motion.div>
        </div>
    );
}