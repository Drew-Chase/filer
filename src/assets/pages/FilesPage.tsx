import FileTable from "../components/FileTable.tsx";
import FileTableBreadcrumbs from "../components/FileTableBreadcrumbs.tsx";
import {useFileSystemEntry} from "../providers/FileSystemEntryProvider.tsx";
import {DirectoryActions} from "../components/DirectoryActions.tsx";
import {motion} from "framer-motion";
import TableFind from "../components/TableFind.tsx";
import FavoritesPanel from "../components/favorites/FavoritesPanel.tsx";
import {useWindow} from "../providers/WindowProvider.tsx";


export default function FilesPage()
{
    const {currentPath} = useFileSystemEntry();
    const {width} = useWindow();

    return (
        <div className={"flex flex-row gap-4 mx-4"}>
            <FavoritesPanel/>
            <motion.div className="container p-4 h-[calc(100dvh_-_100px)] w-[calc(100vw_-_50px)] mx-auto rounded-lg max-w-[unset] bg-white/5"
                        initial={{opacity: 0, y: -50}}
                        animate={{opacity: 1, y: 0}}
                        transition={{duration: 1, delay: 0.75, type: "spring", ease: "easeInOut"}}
            >
                {currentPath !== null && (
                    <div className={"flex flex-col w-full gap-4"}>
                        <div className={"flex flex-row gap-4 items-center justify-between"}>
                            <FileTableBreadcrumbs paths={decodeURIComponent(currentPath).split("/")}/>
                            <TableFind/>
                            {width > 600 && <DirectoryActions/>}
                        </div>

                        <motion.div className="flex flex-row gap-2"
                                    initial={{opacity: 0, y: -50}}
                                    animate={{opacity: 1, y: 0}}
                                    transition={{duration: 1, delay: .75, type: "spring", ease: "easeInOut"}}
                        >
                            <FileTable/>
                        </motion.div>
                    </div>
                )}
            </motion.div>
        </div>
    );
}

