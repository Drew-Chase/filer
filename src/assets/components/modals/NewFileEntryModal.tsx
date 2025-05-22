import {addToast, Button, Checkbox, Input, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {useFileSystemEntry} from "../../providers/FileSystemEntryProvider.tsx";
import {useState} from "react";
import {FileSystem} from "../../ts/filesystem.ts";

type NewFileEntryProperties = {
    isOpen: boolean;
    onClose: () => void;
};

export default function NewFileEntryModal(props: NewFileEntryProperties)
{
    const {currentPath, refresh} = useFileSystemEntry();
    const [filename, setFilename] = useState<string>("");
    const [isDirectory, setIsDirectory] = useState(false);
    const [isLoading, setIsLoading] = useState(false);
    if (currentPath == null) return null;

    const reset = () =>
    {
        setFilename("");
        setIsDirectory(false);
        setIsLoading(false);
    };

    return (
        <Modal
            isOpen={props.isOpen}
            onClose={props.onClose}
            backdrop={"blur"}
            classNames={{
                base: "bg-gradient-to-tr from-[#1d0a3b] to-[#2f115c]"
            }}
        >
            <ModalContent>
                <ModalHeader>New File or Directory</ModalHeader>
                <ModalBody>
                    <Input
                        label={"Filename"}
                        size={"sm"}
                        autoFocus={true}
                        value={filename}
                        onValueChange={setFilename}
                        endContent={<Icon icon={"gg:rename"} width={18} className={"opacity-50 my-auto"}/>}
                        classNames={{
                            inputWrapper: "bg-white/20 data-[hover]:bg-white/15 group-data-[focus]:bg-white/10 group-data-[focus]:outline-primary outline-2 border-transparent"
                        }}
                    />
                    <div className={"flex flex-row items-center justify-between cursor-pointer hover:bg-white/10 p-2 rounded-md"} onClick={() => setIsDirectory(!isDirectory)}>
                        <p>Create Directory</p>
                        <Checkbox isSelected={isDirectory} onValueChange={setIsDirectory} radius={"full"}/>
                    </div>
                </ModalBody>
                <ModalFooter>
                    <Button
                        onPress={async () =>
                        {
                            setIsLoading(true);
                            try
                            {
                                await FileSystem.createEntry(filename, currentPath, isDirectory);
                            } catch (e: Error | any)
                            {
                                addToast({
                                    title: "Failed to create entry",
                                    description: e.message || e.toString() || "Unknown error occurred while trying to create the entry.",
                                    color: "danger"
                                });
                            }
                            refresh();
                            props.onClose();
                            setIsLoading(false);
                            reset();
                        }}
                        variant={"solid"}
                        color={"secondary"}
                        isLoading={isLoading}
                    >
                        New {isDirectory ? "Directory" : "File"}
                    </Button>
                    <Button onPress={() =>
                    {
                        reset();
                        props.onClose();
                    }} variant={"light"} color={"danger"}>Cancel</Button>
                </ModalFooter>
            </ModalContent>
        </Modal>
    );
}