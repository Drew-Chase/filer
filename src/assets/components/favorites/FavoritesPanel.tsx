import {restrictToParentElement} from "@dnd-kit/modifiers";
import {AnimatePresence, motion} from "framer-motion";
import {Button, cn, Divider, Tooltip} from "@heroui/react";
import {useFavorites} from "../../providers/FavoritesProvider.tsx";
import {FavoriteEntry} from "./FavoriteEntry.tsx";
import {useEffect, useState} from "react";
import {Icon} from "@iconify-icon/react";
import {closestCenter, DndContext, DragEndEvent, KeyboardSensor, PointerSensor, useSensor, useSensors} from "@dnd-kit/core";
import {arrayMove, SortableContext, sortableKeyboardCoordinates, verticalListSortingStrategy} from "@dnd-kit/sortable";
import {useWindow} from "../../providers/WindowProvider.tsx";

export default function FavoritesPanel()
{
    const {favorites, setFavorites} = useFavorites();
    const [isExpanded, setIsExpanded] = useState(localStorage.getItem("favorites-panel-expanded") !== "false");
    const sensor = useSensors(useSensor(PointerSensor), useSensor(KeyboardSensor, {coordinateGetter: sortableKeyboardCoordinates}));
    const {width} = useWindow();

    const handleDragEnd = (event: DragEndEvent) =>
    {
        const {active, over} = event;
        if (active && over && active.id !== over.id)
        {
            setFavorites(prev =>
            {
                const oldIndex = prev.findIndex(item => item.path === active.id);
                const newIndex = prev.findIndex(item => item.path === over.id);
                return arrayMove(favorites, oldIndex, newIndex);
            });
        }
    };


    useEffect(() =>
    {
        localStorage.setItem("favorites-panel-expanded", isExpanded.toString());
    }, [isExpanded]);
    return (
        <AnimatePresence>
            <motion.div
                initial={{opacity: 0, y: -20, width: 0, padding: 0}}
                exit={{opacity: 0, y: -20, width: 0, padding: 0}}
                animate={{opacity: favorites.length === 0 || width < 950 ? 0 : 1, y: 0, width: favorites.length === 0 || width < 950 ? 0 : isExpanded ? 400 : 40, padding: favorites.length === 0 || width < 950 ? 0 : isExpanded ? "16px" : "4px"}}
                transition={{duration: 1, delay: 0, type: "spring", ease: "easeInOut"}}
                className={
                    cn(
                        "rounded-lg",
                        "rounded-lg bg-white/5"
                    )
                }
            >
                <div className={"flex flex-row justify-between items-center"}>
                    <motion.h2
                        className={"text-2xl"}
                        initial={{opacity: 0, width: 0}}
                        animate={{opacity: isExpanded ? 1 : 0, width: isExpanded ? "unset" : 0}}
                        transition={{duration: 1, delay: 0, type: "spring", ease: "easeInOut"}}
                    >
                        Favorites
                    </motion.h2>
                    <Tooltip content={"Toggle Favorites Panel"} className={"pointer-events-none"}>
                        <Button variant={"light"} className={"min-w-0 min-h-0 w-10 h-10"} onPress={() => setIsExpanded(prev => !prev)} aria-label={"Toggle Favorites Panel"}>
                            <Icon icon={"icon-park-solid:right-expand"}/>
                        </Button>
                    </Tooltip>
                </div>
                <Divider orientation={"horizontal"} className={"mb-4 mt-2"}/>

                <motion.div
                    className={"flex flex-col gap-2 overscroll-y-auto"}
                    initial={{opacity: 0}}
                    animate={{opacity: isExpanded ? 1 : 0}}
                >
                    <DndContext
                        sensors={sensor}
                        onDragEnd={handleDragEnd}
                        collisionDetection={closestCenter}
                        modifiers={[restrictToParentElement]}
                    >
                        <SortableContext items={favorites.map(i => i.path)} strategy={verticalListSortingStrategy}>
                            {favorites.map(entry => (
                                <FavoriteEntry key={entry.path} entry={entry}/>
                            ))}
                        </SortableContext>
                    </DndContext>
                </motion.div>

            </motion.div>
        </AnimatePresence>
    );
}

