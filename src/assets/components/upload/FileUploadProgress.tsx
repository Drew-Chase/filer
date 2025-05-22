import {Progress, ScrollShadow} from "@heroui/react";

type FileUploadProgressProps = {
    files: File[]
}
export default function FileUploadProgress(props: FileUploadProgressProps)
{
    return (
        <ScrollShadow className={"flex flex-col gap-2 max-h-50dvh min-h-[200px] overflow-y-auto pr-4"}>
            {props.files.map(file => (
                <div key={file.name} className={"flex flex-row gap-8 items-center justify-between bg-white/5 p-2 rounded-lg shadow-md hover:bg-white/10"}>
                    <span className={"w-full truncate"}>{file.webkitRelativePath || file.name}</span>
                    <Progress minValue={0} maxValue={1} value={0.5} size={"sm"} color={"primary"} className={"w-[200px]"}/>
                </div>
            ))}
        </ScrollShadow>
    );
}