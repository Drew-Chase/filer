import {createContext, ReactNode, useCallback, useContext, useEffect, useState} from "react";
import fs, {FilesystemData, FilesystemEntry} from "../ts/filesystem.ts";
import {useLocation, useNavigate} from "react-router-dom";
import {SortDescriptor} from "@heroui/react";
import {useAuth} from "./AuthProvider.tsx";

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
    copyEntry: (sourcePath: string, destinationPath: string) => Promise<void>;
    moveEntry: (sourcePath: string, destinationPath: string) => Promise<void>;
    deleteEntry: (path: string) => Promise<void>;
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

        // Only fetch data if path actually changed
        if (path !== currentPath)
        {
            setCurrentPath(path);
            setLoading(true);
            setData({parent: null, entries: []});

            fs.getEntries(path)
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
        fs.getEntries(currentPath)
            .then(data =>
            {
                setData(data);
            }).catch(e =>
        {
            console.error("Error getting entries:", e);
            setData({parent: null, entries: []});
        })
            .finally(() =>
            {
                reactNavigate(`/files${currentPath.startsWith("/") ? currentPath : "/" + currentPath}`);
                setLoading(false);
            });
    }, [currentPath]);

    useEffect(() =>
    {
        if (!isLoggedIn) return;
        if (data.entries.length === 0) return;
        let column = sortDescriptor.column.toString() as keyof FilesystemEntry;
        data.entries.sort((a, b) =>
        {
            if (column === "size")
            {
                if (sortDescriptor.direction === "ascending")
                {
                    if (a.is_dir)
                        return -1;
                    if (b.is_dir)
                        return 1;
                } else
                {
                    if (a.is_dir)
                        return 1;
                    if (b.is_dir)
                        return -1;
                }
            }
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
            switch (sortDescriptor.direction)
            {
                case "ascending":
                    return a[column] > b[column] ? 1 : -1;
                case "descending":
                    return a[column] > b[column] ? -1 : 1;
            }
        });
    }, [sortDescriptor, currentPath, data]);

    const search = useCallback(async (_query: string, _currentDirectory: boolean) =>
    {

    }, []);

    const copyEntry = useCallback(async (sourcePath: string, destinationPath: string) => {
        if (!isLoggedIn) return;
        
        try {
            await fs.copyEntry(sourcePath, destinationPath);
            // Refresh the current directory to show the changes
            refresh();
        } catch (error) {
            console.error("Error copying entry:", error);
            throw error;
        }
    }, [isLoggedIn, refresh]);

    const moveEntry = useCallback(async (sourcePath: string, destinationPath: string) => {
        if (!isLoggedIn) return;
        
        try {
            await fs.moveEntry(sourcePath, destinationPath);
            // Refresh the current directory to show the changes
            refresh();
        } catch (error) {
            console.error("Error moving entry:", error);
            throw error;
        }
    }, [isLoggedIn, refresh]);

    const deleteEntry = useCallback(async (path: string) => {
        if (!isLoggedIn) return;
        
        try {
            await fs.deleteEntry(path);
            // Refresh the current directory to show the changes
            refresh();
        } catch (error) {
            console.error("Error deleting entry:", error);
            throw error;
        }
    }, [isLoggedIn, refresh]);

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
            copyEntry,
            moveEntry,
            deleteEntry
        }}>
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