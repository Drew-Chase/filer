import {createContext, ReactNode, useCallback, useContext, useEffect, useState} from "react";
import {FileSystem, FilesystemData, FilesystemEntry} from "../ts/filesystem.ts";
import {useLocation, useNavigate} from "react-router-dom";
import {SortDescriptor} from "@heroui/react";
import {useAuth} from "./AuthProvider.tsx";
import RenameModal from "../components/modals/RenameModal.tsx";
import DeleteModal from "../components/modals/DeleteModal.tsx";
import UploadEntryModal from "../components/modals/UploadEntryModal.tsx";
import $ from "jquery";
import NewFileEntryModal from "../components/modals/NewFileEntryModal.tsx";

interface FileSystemEntryContextType
{
    currentPath: string | null;
    navigate: (path: string) => void;
    data: FilesystemData;
    loading: boolean;
    search: (query: string, currentDirectory: boolean) => void;
    sortDescriptor: SortDescriptor;
    onSortChange: (sortDescriptor: SortDescriptor) => void;
    refresh: () => void;
    openRenameModal: (entry: FilesystemEntry) => void;
    openCopyModal: (entry: FilesystemEntry[]) => void;
    openMoveModal: (entry: FilesystemEntry[]) => void;
    openDeleteModal: (entry: FilesystemEntry[]) => void;
    askDeleteSelectedEntries: () => void;
    askCopyMoveSelectedEntries: () => void;
    askCreateNewFileEntry: () => void;
    askUploadEntry: () => void;
    copyEntry: (sourcePath: string, destinationPath: string) => Promise<void>;
    moveEntry: (sourcePath: string, destinationPath: string) => Promise<void>;
    deleteEntry: (paths: string[]) => Promise<void>;
    selectedEntries: Set<FilesystemEntry>;
    setSelectedEntries: (keys: Set<FilesystemEntry>) => void;
    downloadSelected: () => Promise<void>;
    downloadCurrentDirectory: () => Promise<void>;
    downloadEntry: (entry: FilesystemEntry) => Promise<void>;
}

const FileSystemEntryContext = createContext<FileSystemEntryContextType | undefined>(undefined);

export function FileSystemEntryProvider({children}: { children: ReactNode })
{
    const reactNavigate = useNavigate();
    const {pathname} = useLocation();
    const [loading, setLoading] = useState(true);
    const [currentPath, setCurrentPath] = useState<string | null>(null);
    const [data, setData] = useState<FilesystemData>({parent: null, entries: []});
    const [sortDescriptor, setSortDescriptor] = useState<SortDescriptor>({column: "filename", direction: "ascending"} as SortDescriptor);
    const {isLoggedIn} = useAuth();
    const [currentEntryBeingRenamed, setCurrentEntryBeingRenamed] = useState<FilesystemEntry | null>(null);
    // @ts-ignore
    const [currentEntryBeingMoved, setCurrentEntryBeingMoved] = useState<FilesystemEntry[] | null>(null);
    // @ts-ignore
    const [currentEntryBeingCopied, setCurrentEntryBeingCopied] = useState<FilesystemEntry[] | null>(null);
    const [currentEntryBeingDeleted, setCurrentEntryBeingDeleted] = useState<FilesystemEntry[] | null>(null);
    const [selectedEntries, setSelectedEntries] = useState<Set<FilesystemEntry>>(new Set());
    const [isUploadModalOpen, setIsUploadModalOpen] = useState(false);
    const [isNewFileEntryModalOpen, setIsNewFileEntryModalOpen] = useState(false);


// Modify the useEffect hook that watches pathname
    useEffect(() =>
    {
        if (!isLoggedIn) return;

        let path = pathname
            .replace("/files/", "")
            .replace(/^\//, "")
            .replace(/\\/g, "/")
            .replace(/\/\//g, "/");

        if (!path.endsWith("/"))
            path += "/";
        if (path === "")
            path = "/";

        // Only fetch data if the path has actually changed
        if (path !== currentPath)
        {
            setCurrentPath(path);
            setLoading(true);
            setData({parent: null, entries: []});

            FileSystem.getEntries(path)
                .then(data =>
                {
                    setData(data);
                })
                .catch(e =>
                {
                    console.error("Error getting entries:", e);
                    setData({parent: null, entries: []});
                })
                .finally(() =>
                {
                    setLoading(false);
                });

            const body = $("html").off("dragenter").off("drop")
                .on("dragenter", e =>
                {
                    e.preventDefault();
                    e.stopPropagation();
                    askUploadEntry();
                }).on("drop", e =>
                {
                    e.preventDefault();
                    e.stopPropagation();
                })

            ;
            return () =>
            {
                body.off("dragenter").off("drop");
            };

        }
    }, [pathname, isLoggedIn]);

    // Modify the navigate function to only update the route
    const navigate = useCallback((path: string) =>
    {
        if (!isLoggedIn) return;
        if (path === currentPath || path === "") return;

        reactNavigate(`/files${path.startsWith("/") ? path : "/" + path}`);
    }, [currentPath, isLoggedIn]);

    const refresh = useCallback(() =>
    {
        if (!isLoggedIn) return;
        if (currentPath === null) return;
        console.log("Refreshing");
        setLoading(true);
        setData({parent: null, entries: []});
        FileSystem.getEntries(currentPath)
            .then(data =>
            {
                setData(data);
            })
            .catch(e =>
            {
                console.error("Error getting entries:", e);
                setData({parent: null, entries: []});
            })
            .finally(() =>
            {
                reactNavigate(`/files${currentPath.startsWith("/") ? currentPath : "/" + currentPath}`);
                setLoading(false);
                sortEntries();
            });
    }, [currentPath]);

    const sortEntries = useCallback(() =>
    {
        if (!isLoggedIn) return;
        if (data.entries.length === 0) return;
        let column = sortDescriptor.column.toString() as keyof FilesystemEntry;
        data.entries.sort((a, b) =>
        {
            // First sort by directory/file
            if (a.is_dir !== b.is_dir)
            {
                return a.is_dir ? -1 : 1;
            }

            // Then sort by column
            if (column === "filename")
            {
                switch (sortDescriptor.direction)
                {
                    case "ascending":
                        return a[column].localeCompare(b[column]);
                    case "descending":
                        return b[column].localeCompare(a[column]);
                }
            }

            if (column === "size")
            {
                if (a.is_dir && b.is_dir)
                {
                    return 0;
                }
                if (sortDescriptor.direction === "ascending")
                {
                    return (a[column] || 0) > (b[column] || 0) ? 1 : -1;
                } else
                {
                    return (a[column] || 0) > (b[column] || 0) ? -1 : 1;
                }
            }

            if (a[column] == null || b[column] == null) return 0;
            switch (sortDescriptor.direction)
            {
                case "ascending":
                    return a[column] > b[column] ? 1 : -1;
                case "descending":
                    return a[column] > b[column] ? -1 : 1;
            }
        });
    }, [sortDescriptor, currentPath, data, isLoggedIn]);

    useEffect(() =>
    {
        sortEntries();
    }, [sortDescriptor, currentPath, data, isLoggedIn]);

    const search = useCallback(async (_query: string, _currentDirectory: boolean) =>
    {
        // TODO: Implement global searching
    }, []);

    const copyEntry = useCallback(async (sourcePath: string, destinationPath: string) =>
    {
        if (!isLoggedIn) return;

        try
        {
            await FileSystem.copyEntry(sourcePath, destinationPath);
            // Refresh the current directory to show the changes
            refresh();
        } catch (error)
        {
            console.error("Error copying entry:", error);
            throw error;
        }
    }, [isLoggedIn, refresh]);

    const moveEntry = useCallback(async (sourcePath: string, destinationPath: string) =>
    {
        if (!isLoggedIn) return;

        try
        {
            await FileSystem.moveEntry(sourcePath, destinationPath);
            // Refresh the current directory to show the changes
            refresh();
        } catch (error)
        {
            console.error("Error moving entry:", error);
            throw error;
        }
    }, [isLoggedIn, refresh]);

    const deleteEntry = useCallback(async (path: string[]) =>
    {
        if (!isLoggedIn) return;

        try
        {
            await FileSystem.deleteEntry(path);
            refresh();
        } catch (error)
        {
            console.error("Error deleting entry:", error);
            throw error;
        }
    }, [isLoggedIn, refresh]);

    const openRenameModal = useCallback((entry: FilesystemEntry) =>
    {
        setCurrentEntryBeingRenamed(entry);
    }, []);

    const openCopyModal = useCallback((entry: FilesystemEntry[]) =>
    {
        setCurrentEntryBeingCopied(entry);
    }, []);

    const openMoveModal = useCallback((entry: FilesystemEntry[]) =>
    {
        setCurrentEntryBeingMoved(entry);
    }, []);

    const openDeleteModal = useCallback((entry: FilesystemEntry[]) =>
    {
        setCurrentEntryBeingDeleted(entry);
    }, []);

    const downloadEntry = useCallback(async (entry: FilesystemEntry) =>
    {
        if (!isLoggedIn) return;
        try
        {
            await FileSystem.download(entry);
        } catch (error)
        {
            console.error("Error downloading entry:", error);
            throw error;
        }
    }, [isLoggedIn]);

    const downloadSelected = useCallback(async () =>
    {
        if (!isLoggedIn || selectedEntries.size === 0) return;

        try
        {
            // Get the selected entries from the data
            const entriesToDownload = data.entries.filter(entry =>
                selectedEntries.has(entry)
            );

            if (entriesToDownload.length > 0)
            {
                await FileSystem.download(entriesToDownload);
            }
        } catch (error)
        {
            console.error("Error downloading selected entries:", error);
            throw error;
        }
    }, [isLoggedIn, selectedEntries, data.entries]);

    const downloadCurrentDirectory = useCallback(async () =>
    {
        if (!isLoggedIn || !currentPath) return;

        try
        {
            // Find the current directory entry
            const currentDirEntry: FilesystemEntry = {
                filename: currentPath.split("/").filter(Boolean).pop() || "root",
                path: currentPath,
                size: 0,
                last_modified: new Date(),
                creation_date: new Date(),
                is_dir: true
            };

            await FileSystem.download(currentDirEntry);
        } catch (error)
        {
            console.error("Error downloading current directory:", error);
            throw error;
        }
    }, [isLoggedIn, currentPath]);


    const askDeleteSelectedEntries = useCallback(() =>
    {
        setCurrentEntryBeingDeleted([...selectedEntries]);
    }, [selectedEntries]);

    const askCopyMoveSelectedEntries = useCallback(() =>
    {
    }, [selectedEntries]);
    const askUploadEntry = useCallback(() => setIsUploadModalOpen(true), []);
    const askCreateNewFileEntry = useCallback(() =>
    {
        setIsNewFileEntryModalOpen(true);
    }, []);


    return (
        <FileSystemEntryContext.Provider value={{
            currentPath,
            navigate,
            data,
            loading,
            search,
            sortDescriptor,
            onSortChange: setSortDescriptor,
            refresh,
            openRenameModal,
            openCopyModal,
            openMoveModal,
            openDeleteModal,
            copyEntry,
            moveEntry,
            deleteEntry,
            selectedEntries,
            setSelectedEntries,
            downloadSelected,
            downloadCurrentDirectory,
            downloadEntry,
            askDeleteSelectedEntries,
            askCopyMoveSelectedEntries,
            askUploadEntry,
            askCreateNewFileEntry
        }}>
            <RenameModal entry={currentEntryBeingRenamed} onClose={() => setCurrentEntryBeingRenamed(null)}/>
            <DeleteModal entries={currentEntryBeingDeleted} onClose={() => setCurrentEntryBeingDeleted(null)}/>
            <UploadEntryModal isOpen={isUploadModalOpen} onClose={() => setIsUploadModalOpen(false)}/>
            <NewFileEntryModal isOpen={isNewFileEntryModalOpen} onClose={() => setIsNewFileEntryModalOpen(false)}/>
            {children}
        </FileSystemEntryContext.Provider>
    );
}

export function useFileSystemEntry(): FileSystemEntryContextType
{
    const context = useContext(FileSystemEntryContext);
    if (!context)
    {
        throw new Error("useFileSystemEntry must be used within a FileSystemEntryProvider");
    }
    return context;
}
