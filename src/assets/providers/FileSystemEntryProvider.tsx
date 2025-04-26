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

    useEffect(() =>
    {
        if (!isLoggedIn) return;
        let path = pathname
            .replace("/files/", "")  // Remove the /files/ prefix
            .replace(/^\//, "");     // Remove the leading slash if present
        if (path === "")
            path = "/";
        setCurrentPath(path);
        navigate(path);
    }, [pathname, isLoggedIn]);

    const navigate = useCallback(async (path: string) =>
    {
        if (!isLoggedIn) return;
        if (path === currentPath || path === "") return;
        setLoading(true);
        setData({parent: null, entries: []});
        setCurrentPath(path);
        fs.getEntries(path).then(data =>
        {
            setData(data);
        }).finally(() =>
        {
            reactNavigate(`/files${path.startsWith("/") ? path : "/" + path}`);
            setLoading(false);
        });
    }, [currentPath, isLoggedIn]);

    const refresh = useCallback(() =>
    {
        if (!isLoggedIn) return;
        if (currentPath === null) return;
        console.log("Refreshing");
        setLoading(true);
        setData({parent: null, entries: []});
        fs.getEntries(currentPath).then(data =>
        {
            setData(data);
        }).finally(() =>
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

    return (
        <FileSystemEntryContext.Provider value={{currentPath, navigate, data, loading, search, sortDescriptor, onSortChange: setSortDescriptor, refresh}}>
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