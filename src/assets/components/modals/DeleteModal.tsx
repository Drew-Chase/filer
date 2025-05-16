import {Button, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader} from "@heroui/react";
import {FileSystem, FilesystemEntry} from "../../ts/filesystem.ts";
import {useState} from "react";
import {useFileSystemEntry} from "../../providers/FileSystemEntryProvider.tsx";

type DeleteProperties = {
    entry: FilesystemEntry | null;
    onClose: () => void;
};

export default function DeleteModal(props: DeleteProperties)
{
    const [isLoading, setIsLoading] = useState(false);
    const {refresh} = useFileSystemEntry();
    return (
        <Modal isOpen={props.entry !== null} onClose={props.onClose} backdrop={"blur"}>
            <ModalContent>
                <ModalHeader>Delete</ModalHeader>
                <ModalBody className={"flex flex-row"}>
                    <p>
                        Are you sure you want to delete <span className={"font-bold italic inline"}>{props.entry?.filename}</span> and all of its contents? This action cannot be undone.
                    </p>
                </ModalBody>
                <ModalFooter>
                    <Button isLoading={isLoading} onPress={async () =>
                    {
                        setIsLoading(true);
                        await FileSystem.deleteEntry(props.entry?.path || "");
                        refresh();
                        props.onClose();
                        setIsLoading(false);
                    }}>Delete</Button>
                    <Button variant={"light"} color={"danger"} onPress={props.onClose}>Cancel</Button>
                </ModalFooter>
            </ModalContent>
        </Modal>
    );
}