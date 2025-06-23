import {Button, cn, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader, Spinner, Table, TableBody, TableCell, TableColumn, TableHeader, TableRow} from "@heroui/react";
import {useCallback, useEffect, useState} from "react";
import {Icon} from "@iconify-icon/react";
import {FileSystem, FilesystemData} from "../../ts/filesystem.ts";
import FileTableBreadcrumbs from "../FileTableBreadcrumbs.tsx";

type SelectDirectoryEntryProperties = {
    label?: string;
    isOpen: boolean;
    onClose: (directory: string | null) => void;
    initialPath?: string;
};

export default function SelectDirectoryEntryModal(props: SelectDirectoryEntryProperties)
{
    const [selectedPath, setSelectedPath] = useState<string | null>(null);

    const process = useCallback(async () =>
    {
        props.onClose(selectedPath);

    }, [selectedPath]);

    return (
        <Modal
            isOpen={props.isOpen}
            onClose={() =>
            {
                props.onClose(null);
            }}
            backdrop={"blur"}
            classNames={{
                base: "bg-gradient-to-tr from-[#1d0a3b] to-[#2f115c]"
            }}
            size={"4xl"}
            scrollBehavior={"inside"}
        >
            <ModalContent>
                {onClose => (
                    <>
                        <ModalHeader>
                            Select {props.label ?? "Directory"}
                        </ModalHeader>
                        <ModalBody>
                            <FileTable selectedItem={selectedPath} onSelectionChange={setSelectedPath}/>
                        </ModalBody>
                        <ModalFooter>
                            {selectedPath === null ?
                                <Button color={"secondary"} isDisabled>Select Directory</Button> :
                                <Button onPress={process} color={"secondary"}>Select {selectedPath.replace(/\\/g, "/").split("/").pop()}</Button>
                            }
                            <Button onPress={onClose} variant={"light"} color={"danger"}>Cancel</Button>
                        </ModalFooter>
                    </>
                )}
            </ModalContent>
        </Modal>
    );
}

type FileTableProperties = {
    selectedItem: string | null;
    onSelectionChange: (value: string | null) => void;
    initialPath?: string;
}

function FileTable(props: FileTableProperties)
{
    const [isLoading, setIsLoading] = useState(false);
    const [currentPath, setCurrentPath] = useState<string>(props.initialPath ?? "/");
    const [data, setData] = useState<FilesystemData>({parent: null, entries: []});

    useEffect(() =>
    {
        setIsLoading(true);
        FileSystem
            .getEntries(currentPath)
            .then(data =>
            {
                setData({parent: data.parent, entries: data.entries.filter(i => i.is_dir)});
            })
            .finally(() =>
            {
                props.onSelectionChange(currentPath);
                setIsLoading(false);
            });
    }, [currentPath]);

    useEffect(() =>
    {
        console.log("Selected item changed:", props.selectedItem);
    }, [props.selectedItem]);

    return (
        <>
            <FileTableBreadcrumbs paths={currentPath.replace(/\\/g, "/").split("/")} onNavigate={setCurrentPath}/>
            <Table
                hideHeader
                removeWrapper
                isHeaderSticky
                aria-label={"File table"}
                className={"w-full"}
                classNames={{
                    base: "w-full max-h-[calc(100dvh_-_180px)] overflow-y-auto",
                    td: "group-aria-[selected=false]/tr:group-data-[hover=true]/tr:before:bg-white/10 before:bg-white/10 before:transition-all before:duration-200",
                    tr: cn(
                        "data-[selected=true]:!bg-blue-500/10 data-[selected=true]:data-[hover=true]:!bg-blue-500/20 duration-200 transition-all"
                    )
                }}
                selectionMode={"single"}
                selectedKeys={new Set(props.selectedItem ? [props.selectedItem] : [])}
                onSelectionChange={keys =>
                {
                    // Convert the keys to a Set of strings
                    const selectedKeys = new Set(Array.from(keys).map(key =>
                    {
                        // Find the entry with this key (index)
                        const path = key.toString();
                        return data.entries.find(i => i.path === path) ?? null;
                    }).filter(path => path != null));

                    props.onSelectionChange(selectedKeys.size === 1 ? selectedKeys.values().next().value?.path ?? null : null);
                }}
            >
                <TableHeader>
                    <TableColumn key={"filename"} className="w-full" allowsSorting aria-label="Name column">Name</TableColumn>
                </TableHeader>
                <TableBody isLoading={isLoading} loadingContent={<Spinner color={"primary"}/>}>
                    {data.entries.map((entry) => (
                        <TableRow key={entry.path} aria-label={`File entry: ${entry.filename}`}>
                            <TableCell className="font-medium" aria-label="File name" onDoubleClick={() =>
                            {
                                if (entry.is_dir)
                                    setCurrentPath(entry.path);
                            }}>
                                <Button
                                    onPress={() =>
                                    {
                                        if (entry.is_dir)
                                            setCurrentPath(entry.path);
                                    }}
                                    variant={"light"}
                                    size={"sm"}
                                    className={`text-start justify-start`}
                                    aria-label={`Open directory ${entry.filename}`}
                                >
                                    <Icon
                                        icon={entry.is_dir ? "mage:folder-fill" : "mage:file-fill"}
                                        className={"text-2xl data-[directory=true]:text-blue-500"}
                                        data-directory={entry.is_dir ? "true" : "false"}
                                        aria-hidden="true"
                                    />
                                    {entry.filename}
                                </Button>
                            </TableCell>
                        </TableRow>
                    ))}

                    {data.entries.length === 0 && !isLoading && (
                        <TableRow aria-label={"No files found"}>
                            <TableCell className="text-center py-8" aria-label="No files found">
                                {data?.entries.length === 0 ? "This directory is empty" : "No matching files found"}
                            </TableCell>
                        </TableRow>
                    ) as any}
                </TableBody>
            </Table>
        </>
    );
}
