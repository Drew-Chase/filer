import {Progress, ScrollShadow} from "@heroui/react";
import {useEffect, useState} from "react";
import {useFileSystemEntry} from "../../providers/FileSystemEntryProvider.tsx";
import {FileSystem} from "../../ts/filesystem.ts";

type FileUploadProgressProps = {
    files: File[];
    onUploadSpeedChange: (speed: number) => void;
    onUploadComplete: () => void;
}
export default function FileUploadProgress(props: FileUploadProgressProps)
{
    const {currentPath} = useFileSystemEntry();
    const [fileProgresses, setFileProgresses] = useState<Map<File, number>>(new Map());
    useEffect(() =>
    {
        (async () =>
        {
            if (currentPath == null) return;
            const files = [...props.files];

            let lastUpdateTime = Date.now();
            let lastBytes = 0;

            for (const file of files)
            {
                const totalBytes = file.size;
                await FileSystem.upload(file, currentPath, bytes =>
                {
                    const progress = bytes / totalBytes;
                    setFileProgresses(prev =>
                    {
                        prev.set(file, progress);
                        return new Map(prev);
                    });

                    const now = Date.now();
                    const timeDiff = (now - lastUpdateTime) / 1000; // Convert to seconds
                    if (timeDiff >= 1)
                    { // Update speed every second
                        const bytesDiff = bytes - lastBytes;
                        props.onUploadSpeedChange(Math.round(bytesDiff / timeDiff));
                        lastUpdateTime = now;
                        lastBytes = bytes;
                    }
                });
                props.onUploadSpeedChange(0);
            }

            props.onUploadSpeedChange(0);
            props.onUploadComplete();
        })(); // Self-invoking function
    }, [props.files]);
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