import {addToast, Button, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader} from "@heroui/react";
import {FilesystemEntry} from "../../ts/filesystem.ts";
import {useState} from "react";
import {useFileSystemEntry} from "../../providers/FileSystemEntryProvider.tsx";

type DeleteProperties = {
    entries: FilesystemEntry[] | null;
    onClose: () => void;
};

export default function DeleteModal(props: DeleteProperties)
{
    const [isLoading, setIsLoading] = useState(false);
    const {deleteEntry} = useFileSystemEntry();
    return (
        <Modal
            isOpen={props.entries !== null}
            onClose={props.onClose}
            backdrop={"blur"}
            classNames={{
                base: "bg-gradient-to-tr from-[#1d0a3b] to-[#2f115c]"
            }}
        >
            <ModalContent>
                <ModalHeader>Delete {props.entries?.length} Items</ModalHeader>
                <ModalBody className={"flex flex-row"}>
                    <p>
                        Are you sure you want to delete
                        {props.entries?.length === 1 ?
                            <span className={"font-bold italic inline mx-1"}>{props.entries[0].filename}</span> :
                            <span className={"font-bold italic inline mx-1"}>{props.entries?.length} Entries</span>
                        }
                        and all of their contents? This action cannot be undone.
                    </p>
                </ModalBody>
                <ModalFooter>
                    <Button isLoading={isLoading} color={"secondary"} onPress={async () =>
                    {
                        setIsLoading(true);
                        if (props.entries === null)
                            addToast({
                                title: "Error",
                                description: "No entries selected"
                            });
                        else
                        {
                            try
                            {
                                await deleteEntry(props.entries.map(i => i.path));
                            } catch (e: Error | any)
                            {
                                addToast({
                                    title: "Error",
                                    description: e.message || e.toString() || "Unknown error occurred while trying to delete the entry.",
                                    color: "danger"
                                });
                            }
                        }
                        props.onClose();
                        setIsLoading(false);
                    }}>Delete</Button>
                    <Button variant={"light"} color={"danger"} onPress={props.onClose}>Cancel</Button>
                </ModalFooter>
            </ModalContent>
        </Modal>
    );
}