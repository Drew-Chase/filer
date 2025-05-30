import {Progress, ScrollShadow} from "@heroui/react";
import {useEffect, useState, useRef} from "react";
import {useFileSystemEntry} from "../../providers/FileSystemEntryProvider.tsx";
import {FileSystem} from "../../ts/filesystem.ts";

type FileUploadProgressProps = {
    files: File[];
    onUploadSpeedChange: (speed: number) => void;
    onUploadComplete: () => void;
    onCancelUpload?: (file: File) => void;
    registerCancelFunction?: (file: File, cancelFn: () => Promise<void>) => void;
}
export default function FileUploadProgress(props: FileUploadProgressProps)
{
    const {currentPath} = useFileSystemEntry();
    const [fileProgresses, setFileProgresses] = useState<Map<File, number>>(new Map());
    const [cancelledFiles, setCancelledFiles] = useState<Set<File>>(new Set());
    const activeUploadsRef = useRef<Map<File, { cancel: () => Promise<void> }>>(new Map());

    useEffect(() =>
    {
        (async () =>
        {
            if (currentPath == null) return;
            const files = [...props.files];

            let lastUpdateTime = Date.now();
            let lastBytes = 0;
            let completedFiles = 0;

            for (const file of files)
            {
                if (cancelledFiles.has(file)) {
                    completedFiles++;
                    continue;
                }

                const totalBytes = file.size;
                const { promise, cancel } = await FileSystem.upload(
                    file, 
                    currentPath, 
                    bytes => {
                        const progress = bytes / totalBytes;
                        setFileProgresses(prev => {
                            prev.set(file, progress);
                            return new Map(prev);
                        });

                        const now = Date.now();
                        const timeDiff = (now - lastUpdateTime) / 1000; // Convert to seconds
                        if (timeDiff >= 1) { // Update speed every second
                            const bytesDiff = bytes - lastBytes;
                            props.onUploadSpeedChange(Math.round(bytesDiff / timeDiff));
                            lastUpdateTime = now;
                            lastBytes = bytes;
                        }
                    },
                    () => {
                        // Handle cancellation
                        setCancelledFiles(prev => {
                            const newSet = new Set(prev);
                            newSet.add(file);
                            return newSet;
                        });
                        activeUploadsRef.current.delete(file);

                        // Mark as complete with a special progress value
                        setFileProgresses(prev => {
                            prev.set(file, -1); // Use -1 to indicate canceled
                            return new Map(prev);
                        });
                    }
                );

                // Store the cancel function
                activeUploadsRef.current.set(file, { cancel });

                // Register the cancel function with the parent component if needed
                if (props.registerCancelFunction) {
                    props.registerCancelFunction(file, cancel);
                }

                try {
                    await promise;
                    completedFiles++;
                    activeUploadsRef.current.delete(file);
                } catch (error) {
                    console.error(`Error uploading ${file.name}:`, error);
                    activeUploadsRef.current.delete(file);
                    completedFiles++;
                }

                props.onUploadSpeedChange(0);

                // If all files are completed, call onUploadComplete
                if (completedFiles >= files.length) {
                    props.onUploadSpeedChange(0);
                    props.onUploadComplete();
                }
            }
        })(); // Self-invoking function

        // Cleanup function to cancel any active uploads when component unmounts
        return () => {
            activeUploadsRef.current.forEach(({ cancel }) => {
                cancel().catch(err => console.error("Error cancelling upload during cleanup:", err));
            });
        };
    }, [props.files, cancelledFiles]);
    return (
        <ScrollShadow className={"flex flex-col gap-2 max-h-50dvh min-h-[200px] overflow-y-auto pr-4"}>
            {
                props.files
                    .sort((a, b) =>
                    {
                        const aProgress = fileProgresses.get(a) ?? 0;
                        const bProgress = fileProgresses.get(b) ?? 0;
                        if (aProgress === 1) return 1;
                        if (bProgress === 1) return -1;
                        if (aProgress > 0 && aProgress < 1) return -1;
                        if (bProgress > 0 && bProgress < 1) return 1;
                        return 0;
                    })
                    .map(file => (
                        <div key={file.name} className={"flex flex-row gap-8 items-center justify-between bg-white/5 p-2 rounded-lg shadow-md hover:bg-white/10"}>
                            <span className={"w-full truncate"}>{file.webkitRelativePath || file.name}</span>
                            <Progress minValue={0} maxValue={1} value={fileProgresses.get(file) ?? 0} size={"sm"} color={"primary"} className={"w-[200px]"}/>
                        </div>
                    ))}
        </ScrollShadow>
    );
}
