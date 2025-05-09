import {Alert, Button, ButtonGroup, CircularProgress, Tooltip} from "@heroui/react";
import fs from "../ts/filesystem.ts";
import {Icon} from "@iconify-icon/react";
import {useState} from "react";
import {useFileSystemEntry} from "../providers/FileSystemEntryProvider.tsx";

type FileUploadData = {
    name: string;
    progress: number;
}

export function DirectoryActions()
{
    const {currentPath, refresh, loading} = useFileSystemEntry();
    const [fileUploadData, setFileUploadData] = useState<FileUploadData | null>(null);
    if (currentPath === null) return <></>;

    const handleUpload = async () =>
    {
        const input = document.createElement("input");
        input.type = "file";
        input.multiple = false;
        input.click();
        input.onchange = async () =>
        {
            const file = input.files?.[0];
            if (!file)
                return;
            await fs.upload(file, currentPath, bytes =>
            {
                setFileUploadData({name: file.name, progress: bytes / file.size});
            });
            setTimeout(() =>
            {
                setFileUploadData(null);
            }, 5 * 1000);
        };
    };

    return (
        <>
            <ButtonGroup>
                <Tooltip content={"Upload File"}><Button variant={"ghost"} size={"sm"} className={"text-xl"} onPress={handleUpload}><Icon icon={"mage:file-upload-fill"}/></Button></Tooltip>
                <Tooltip content={"Archive and Download"}><Button size={"sm"} variant={"ghost"} className={"text-xl"}><Icon icon={"mage:archive-fill"}/></Button></Tooltip>
                <Tooltip content={"Create New Directory"}><Button size={"sm"} variant={"ghost"} className={"text-xl"}><Icon icon={"mage:folder-plus-fill"}/></Button></Tooltip>
                <Tooltip content={"Create New File"}><Button size={"sm"} variant={"ghost"} className={"text-xl"}><Icon icon={"mage:file-plus-fill"}/></Button></Tooltip>
                <Tooltip content={"Refresh"}><Button size={"sm"} variant={"ghost"} className={"text-xl"} onPress={refresh} isLoading={loading}>{!loading && <Icon icon={"mage:refresh"}/>}</Button></Tooltip>
            </ButtonGroup>
            {fileUploadData && (
                <Alert
                    className={"absolute w-fit bottom-5 right-5"}
                    classNames={{base: "bg-primary/10 backdrop-blur-sm"}}
                    variant={"solid"}
                    icon={<CircularProgress minValue={0} maxValue={1} value={fileUploadData.progress} color={"primary"}/>}
                    title={
                        <Tooltip content={`Uploading: ${fileUploadData.name}`} delay={1000}>
                            <p className={"max-w-[250px] truncate"}>Uploading: {fileUploadData.name}</p>
                        </Tooltip>
                    }
                    description={`Uploading file ${(fileUploadData.progress * 100).toFixed(2)}%...`}
                />
            )}
        </>
    );
}
