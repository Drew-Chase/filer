import {extensionFileTypeMap} from "./file-type-match.ts";

/**
 * Represents a filesystem entry (file or directory)
 */
export interface FilesystemEntry
{
    filename: string;
    path: string;
    size: number;
    last_modified: Date;
    creation_date: Date;
    is_dir: boolean;
    file_type?: string;
}

/**
 * Represents a directory listing with entries and parent path
 */
export interface FilesystemData
{
    parent: string | null;
    entries: FilesystemEntry[];
}

/**
 * FileSystem class for handling filesystem operations
 * Provides methods to browse directories and download files
 */
export class FileSystem
{

    /**
     * Get filesystem entries for the specified path
     * @param path Directory path to browse
     * @returns Promise with the filesystem data
     */
    static async getEntries(path: string): Promise<FilesystemData>
    {
        try
        {
            const response = await fetch(`/api/filesystem/`, {
                headers: {
                    "X-Filesystem-Path": decodeURIComponent(path)
                }
            });

            if (!response.ok)
            {
                throw new Error(`Error: ${response.status} - ${response.statusText}`);
            }

            let tmp = await response.json() as FilesystemData;

            tmp.entries.map(entry =>
            {
                if ((entry as any).created)
                {
                    const createdSecs = ((entry as any).created.secs_since_epoch || 0) * 1000;
                    const createdNanos = ((entry as any).created.nanos_since_epoch || 0) / 1_000_000;
                    entry.creation_date = new Date(createdSecs + createdNanos);
                }

                if ((entry as any).last_modified)
                {
                    const modifiedSecs = ((entry as any).last_modified.secs_since_epoch || 0) * 1000;
                    const modifiedNanos = ((entry as any).last_modified.nanos_since_epoch || 0) / 1_000_000;
                    entry.last_modified = new Date(modifiedSecs + modifiedNanos);
                }

                if (entry.is_dir)
                {
                    entry.file_type = "Folder";
                } else
                {
                    const extensions = entry.filename.toLowerCase().trim().split(".").slice(1);
                    let extension = extensions.length > 0 ? extensions.join(".") : "";
                    entry.file_type = extensionFileTypeMap.find(e => e.extensions.includes(extension))?.description ?? "File";
                }

                return entry;
            });
            return tmp;
        } catch (error)
        {
            console.error("Error fetching filesystem data:", error);
            throw error;
        }
    }

    /**
     * Download a file or directory
     * @param entry Filesystem entry to download
     * @returns Promise that resolves when download is initiated
     */
    static async download(entry: FilesystemEntry | FilesystemEntry[]): Promise<void>
    {
        const cwd = window.location.pathname.replace("/files/", "");
        const url = new URL(`/api/filesystem/download`, window.location.href);

        const items = entry instanceof Array ? entry : [entry];
        url.searchParams.set("items", JSON.stringify(items.map(e => e.path.replace(cwd, ""))));

        url.searchParams.set("cwd", cwd);

        const anchor = document.createElement("a");
        // anchor.target = "_blank";
        anchor.href = url.href;
        anchor.click();
    }


    static async copyEntry(sourcePath: string, destinationPath: string): Promise<void>
    {
        const response = await fetch("/api/filesystem/copy", {
            method: "POST",
            headers: {
                "X-Filesystem-Path": sourcePath,
                "X-NewFilesystem-Path": destinationPath
            }
        });

        if (!response.ok)
        {
            const errorData = await response.json();
            throw new Error(errorData.error || `Failed to copy: ${response.statusText}`);
        }
    }

    static async moveEntry(sourcePath: string, destinationPath: string): Promise<void>
    {
        const response = await fetch("/api/filesystem/move", {
            method: "POST",
            headers: {
                "X-Filesystem-Path": sourcePath,
                "X-NewFilesystem-Path": destinationPath
            }
        });

        if (!response.ok)
        {
            const errorData = await response.json();
            throw new Error(errorData.error || `Failed to move: ${response.statusText}`);
        }
    }

    static async deleteEntry(path: string | string[]): Promise<void>
    {
        const response = await fetch("/api/filesystem/", {
            method: "DELETE",
            headers: {
                "X-Filesystem-Paths": JSON.stringify(path instanceof Array ? path : [path])
            }
        });

        if (!response.ok)
        {
            const errorData = await response.json();
            throw new Error(errorData.error || `Failed to delete: ${response.statusText}`);
        }
    }

    /**
     * Format file size into human-readable format
     * @param bytes Size in bytes
     * @returns Formatted size string (e.g., "2.5 MB")
     */
    public static formatSize(bytes: number): string
    {
        if (bytes === 0) return "0 Bytes";

        const k = 1024;
        const sizes = ["Bytes", "KB", "MB", "GB", "TB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));

        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
    }

    /**
     * Check if a path exists
     * @param path Path to check
     * @returns Promise indicating if the path exists
     */
    public static async pathExists(path: string): Promise<boolean>
    {
        try
        {
            await FileSystem.getEntries(path);
            return true;
        } catch (error)
        {
            return false;
        }
    }

    /**
     * Get file or directory information
     * @param path Path to the file or directory
     * @returns Promise with the filesystem entry
     */
    public static async getInfo(path: string): Promise<FilesystemEntry | null>
    {
        try
        {
            const dirname = FileSystem.getDirectoryName(path);
            const filename = FileSystem.getFileName(path);

            const data = await FileSystem.getEntries(dirname);
            return data.entries.find(entry => entry.filename === filename) || null;
        } catch (error)
        {
            console.error("Error getting file info:", error);
            return null;
        }
    }

    /**
     * Get the directory name from a path
     * @param path Full path
     * @returns Directory path
     */
    private static getDirectoryName(path: string): string
    {
        const lastSlashIndex = path.lastIndexOf("/");
        if (lastSlashIndex <= 0) return "/";
        return path.substring(0, lastSlashIndex);
    }

    /**
     * Get the file name from a path
     * @param path Full path
     * @returns File name
     */
    private static getFileName(path: string): string
    {
        const lastSlashIndex = path.lastIndexOf("/");
        return path.substring(lastSlashIndex + 1);
    }

    public static async upload(file: File, path: string, updateProgress: (bytes: number) => void): Promise<void>
    {
        // Generate unique upload ID
        const uploadId = crypto.randomUUID();

        return new Promise<void>((resolve, reject) =>
        {
            // Set up the SSE listener for progress
            const events = new EventSource(`/api/filesystem/upload/progress/${uploadId}`);

            events.onmessage = (event) =>
            {
                const data = JSON.parse(event.data);
                switch (data.status)
                {
                    case "progress":
                        console.log(`Upload progress: ${data.bytesUploaded} bytes`);
                        updateProgress(data.bytesUploaded);
                        break;
                    case "complete":
                        console.log(`Upload complete: ${data.bytesUploaded} bytes`);
                        events.close();
                        resolve();
                        break;
                    case "error":
                        events.close();
                        reject(new Error(data.message));
                        break;
                }
            };

            events.onerror = () =>
            {
                events.close();
                reject(new Error("EventSource connection failed"));
            };

            events.onopen = () =>
            {
                // Start the upload once connected
                fetch(`/api/filesystem/upload`, {
                    method: "POST",
                    headers: {
                        "X-Filesystem-Path": `${path}/${file.name}`,
                        "X-Upload-ID": uploadId
                    },
                    body: file
                }).then(response =>
                {
                    if (!response.ok)
                    {
                        events.close();
                        reject(new Error(`Upload failed: ${response.status} - ${response.statusText}`));
                    }
                }).catch(error =>
                {
                    events.close();
                    reject(error);
                });
            };
        });
    }
}
