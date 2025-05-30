import {Button, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader, Spinner, Tab, Table, TableBody, TableCell, TableColumn, TableHeader, TableRow, Tabs} from "@heroui/react";
import {useCallback, useEffect, useState} from "react";
import {Icon} from "@iconify-icon/react";
import {FileSystem, FilesystemData} from "../../ts/filesystem.ts";
import {useFileSystemEntry} from "../../providers/FileSystemEntryProvider.tsx";
import FileTableBreadcrumbs from "../FileTableBreadcrumbs.tsx";

type CopyMoveEntryProperties = {
    isOpen: boolean;
    onClose: () => void;
};

export default function CopyMoveEntryModal(props: CopyMoveEntryProperties)
{
    const [isMove, setIsMove] = useState(true);
    const [selectedPath, setSelectedPath] = useState<string | null>(null);
    const {currentPath, selectedEntries, refresh} = useFileSystemEntry();

    const process = useCallback(async () =>
    {
        if (selectedPath === null || selectedPath.replace(/\\/g, "/").trim().replace(/\/+$/, "") === currentPath?.replace(/\\/g, "/")?.trim().replace(/\/+$/, "")) return;
        if (currentPath == null) return;
        if (isMove)
        {
            await FileSystem.moveEntry([...selectedEntries].map(i => i.path), selectedPath);
        } else
        {
            await FileSystem.copyEntry([...selectedEntries].map(i => i.path), selectedPath);
        }

        refresh();
        props.onClose();

    }, [selectedPath, isMove]);

    return (
        <Modal
            isOpen={props.isOpen}
            onClose={props.onClose}
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
                            <Tabs
                                size={"lg"}
                                variant={"underlined"}
                                onSelectionChange={index =>
                                {
                                    setIsMove(index != "copy");
                                }}
                                selectedKey={isMove ? "move" : "copy"}
                            >
                                <Tab key={"move"} title={"Move Entry"}/>
                                <Tab key={"copy"} title={"Copy Entry"}/>
                            </Tabs>
                        </ModalHeader>
                        <ModalBody>
                            <FileTable selectedItem={selectedPath} onSelectionChange={setSelectedPath}/>
                        </ModalBody>
                        <ModalFooter>
                            {selectedPath === null || selectedPath.replace(/\\/g, "/").trim().replace(/\/+$/, "") === currentPath?.replace(/\\/g, "/")?.trim().replace(/\/+$/, "") ?
                                <Button color={"secondary"} isDisabled>{isMove ? "Move" : "Copy"}</Button> :
                                <Button onPress={process} color={"secondary"}>{isMove ? "Move" : "Copy"} to {selectedPath.replace(/\\/g, "/").split("/").pop()}</Button>
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
}

function FileTable(props: FileTableProperties)
{
    const {currentPath: pathname} = useFileSystemEntry();
    const [isLoading, setIsLoading] = useState(false);
    const [currentPath, setCurrentPath] = useState<string>(pathname ?? "/");
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

    return (
        <>
            <FileTableBreadcrumbs paths={currentPath.replace(/\\/g, "/").split("/")} onNavigate={path => setCurrentPath(path)}/>
            <Table
                hideHeader
                removeWrapper
                isHeaderSticky
                aria-label={"File table"}
                className={"w-full"}
                classNames={{
                    base: "w-full max-h-[calc(100dvh_-_180px)] overflow-y-auto",
                    th: "!bg-white/10 backdrop-contrast-105 backdrop-brightness-75 backdrop-blur-lg",
                    td: "group-aria-[selected=false]/tr:group-data-[hover=true]/tr:before:bg-white/10 before:bg-white/10 before:transition-all before:duration-200"
                }}
                selectionMode={"single"}
                selectedKeys={new Set(props.selectedItem ? [props.selectedItem] : [])}
                onSelectionChange={keys =>
                {
                    // Convert the keys to a Set of strings
                    const selectedKeys = new Set(Array.from(keys).map(key =>
                    {
                        // Find the entry with this key (index)
                        const index = parseInt(key.toString());
                        const path = data.entries[index]?.path;
                        if (isNaN(index) || path === undefined) return null;

                        return data.entries[index] ?? null;
                    }).filter(path => path != null));

                    props.onSelectionChange(selectedKeys.size === 1 ? selectedKeys.values().next().value?.path ?? null : null);
                }}
            >
                <TableHeader>
                    <TableColumn key={"filename"} className="w-full" allowsSorting aria-label="Name column">Name</TableColumn>
                </TableHeader>
                <TableBody isLoading={isLoading} loadingContent={<Spinner color={"primary"}/>}>
                    {data.entries.map((entry, index) => (
                        <TableRow key={index} aria-label={`File entry: ${entry.filename}`}>
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
