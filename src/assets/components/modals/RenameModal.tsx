import {Button, Input, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader} from "@heroui/react";
import {FileSystem, FilesystemEntry} from "../../ts/filesystem.ts";
import {Icon} from "@iconify-icon/react";
import {useEffect, useState} from "react";
import {useFileSystemEntry} from "../../providers/FileSystemEntryProvider.tsx";

type RenameProperties = {
    entry: FilesystemEntry | null;
    onClose: () => void;
};

export default function RenameModal(props: RenameProperties)
{
    const {refresh} = useFileSystemEntry();
    const [filename, setFilename] = useState<string>(props.entry?.filename || "");
    const [isLoading, setIsLoading] = useState(false);


    useEffect(() =>
    {
        if (props.entry === null) return;
        setFilename(props.entry.filename);
    }, [props.entry]);

    return (
        <Modal
            isOpen={props.entry !== null}
            onClose={props.onClose}
            classNames={{
                base: "bg-gradient-to-tr from-[#1d0a3b] to-[#2f115c]"
            }}
            backdrop={"blur"}
        >
            <ModalContent>
                {onClose => (
                    <>
                        <ModalHeader>Rename</ModalHeader>
                        <ModalBody>
                            <Input
                                label={"Filename"}
                                size={"sm"}
                                autoFocus={true}
                                value={filename}
                                onValueChange={setFilename}
                                endContent={<Icon icon={"gg:rename"} width={18} className={"opacity-50 my-auto"}/>}
                                classNames={{
                                    inputWrapper: "bg-white/20 data-[hover]:bg-white/15 group-data-[focus]:bg-white/10 group-data-[focus]:border-primary border-1 border-transparent"
                                }}
                            />
                        </ModalBody>
                        <ModalFooter>
                            <Button
                                color={"secondary"}
                                isLoading={isLoading}
                                onPress={async () =>
                                {
                                    setIsLoading(true);
                                    if (props.entry?.filename === filename)
                                    {
                                        onClose();
                                        setIsLoading(false);
                                        return;
                                    }
                                    let oldPath = props.entry?.path ?? "";
                                    let newFilePath = `${oldPath.substring(0, oldPath.lastIndexOf("/"))}/${filename}`;
                                    await FileSystem.renameEntry(oldPath, newFilePath);
                                    refresh();
                                    onClose();
                                    setIsLoading(false);
                                }}
                            >
                                Rename
                            </Button>
                            <Button onPress={onClose} variant={"light"} color={"danger"}>Cancel</Button>
                        </ModalFooter>
                    </>
                )}
            </ModalContent>
        </Modal>
    );
}