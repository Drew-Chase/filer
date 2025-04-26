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
     * Base URL for filesystem API endpoints
     */
    private readonly baseUrl: string = "/api/filesystem";

    /**
     * Get filesystem entries for the specified path
     * @param path Directory path to browse
     * @returns Promise with the filesystem data
     */
    public async getEntries(path: string): Promise<FilesystemData>
    {
        try
        {
            const response = await fetch(`${this.baseUrl}/`, {
                headers: {
                    "X-Filesystem-Path": path
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
    public async download(entry: FilesystemEntry): Promise<void>
    {
        try
        {
            const response = await fetch(`${this.baseUrl}/download`, {
                headers: {
                    "X-Filesystem-Path": entry.path
                }
            });

            if (!response.ok)
            {
                throw new Error(`Download failed: ${response.status} - ${response.statusText}`);
            }

            const blob = await response.blob();
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement("a");
            a.style.display = "none";
            a.href = url;
            a.download = entry.filename;
            document.body.appendChild(a);
            a.click();
            window.URL.revokeObjectURL(url);
            document.body.removeChild(a);
        } catch (error)
        {
            console.error("Download error:", error);
            throw error;
        }
    }

    /**
     * Format file size into human-readable format
     * @param bytes Size in bytes
     * @returns Formatted size string (e.g., "2.5 MB")
     */
    public formatSize(bytes: number): string
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
    public async pathExists(path: string): Promise<boolean>
    {
        try
        {
            await this.getEntries(path);
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
    public async getInfo(path: string): Promise<FilesystemEntry | null>
    {
        try
        {
            const dirname = this.getDirectoryName(path);
            const filename = this.getFileName(path);

            const data = await this.getEntries(dirname);
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
    private getDirectoryName(path: string): string
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
    private getFileName(path: string): string
    {
        const lastSlashIndex = path.lastIndexOf("/");
        return path.substring(lastSlashIndex + 1);
    }

    public async upload(file: File, path: string, updateProgress: (bytes: number) => void): Promise<void>
    {
        // Generate unique upload ID
        const uploadId = crypto.randomUUID();

        return new Promise<void>((resolve, reject) =>
        {
            // Set up SSE listener for progress
            const events = new EventSource(`${this.baseUrl}/upload/progress/${uploadId}`);

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
                fetch(`${this.baseUrl}/upload`, {
                    method: "POST",
                    headers: {
                        "X-Filesystem-Path": `${path}/${file.name}+test`,
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

// Create a default instance for easy import
const fileSystem = new FileSystem();
export default fileSystem;