import {Button, Input, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {useFileSystemEntry} from "../../providers/FileSystemEntryProvider.tsx";
import {useState} from "react";

type NewFileEntryProperties = {
    isOpen: boolean;
    onClose: () => void;
};

export default function NewFileEntryModal(props: NewFileEntryProperties)
{
    const {} = useFileSystemEntry();
    const [filename, setFilename] = useState<string>("");
    const [isLoading, setIsLoading] = useState(false);
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
                {onClose => (
                    <>
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
                        </ModalBody>
                        <ModalFooter>
                            <Button onPress={onClose} variant={"solid"} color={"secondary"}>New File</Button>
                            <Button onPress={onClose} variant={"ghost"} color={"secondary"}>New Directory</Button>
                            <Button onPress={onClose} variant={"light"} color={"danger"}>Cancel</Button>
                        </ModalFooter>
                    </>
                )}
            </ModalContent>
        </Modal>
    );
}