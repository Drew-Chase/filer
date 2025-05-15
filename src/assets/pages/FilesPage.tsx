import FileTable from "../components/FileTable.tsx";
import FileTableBreadcrumbs from "../components/FileTableBreadcrumbs.tsx";
import {useFileSystemEntry} from "../providers/FileSystemEntryProvider.tsx";
import {DirectoryActions} from "../components/DirectoryActions.tsx";


export default function FilesPage()
{
    const {currentPath} = useFileSystemEntry();

    return (
        <div className="container p-4 h-[calc(100dvh_-_100px)] w-[calc(100vw_-_50px)] mx-auto rounded-lg max-w-[unset] bg-white/5">
            {currentPath !== null && (
                <div className={"flex flex-col w-full gap-4"}>
                    <div className={"flex flex-row gap-4 items-center justify-between"}>
                        <FileTableBreadcrumbs paths={decodeURIComponent(currentPath).split("/")}/>
                        <DirectoryActions/>
                    </div>
                    <FileTable/>
                </div>
            )}
        </div>
    );
}

