import {Button, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader} from "@heroui/react";
import {FilesystemEntry} from "../../ts/filesystem.ts";

type DeleteProperties = {
    entry: FilesystemEntry | null;
    onClose: (confirm: boolean) => void;
};

export default function DeleteModal(props: DeleteProperties)
{
    return (
        <Modal isOpen={props.entry !== null}>
            <ModalContent>
                <ModalHeader>Delete</ModalHeader>
                <ModalBody>
                    Are you sure you want to delete this?
                </ModalBody>
                <ModalFooter>
                    <Button color={"primary"} onPress={() => props.onClose(true)}>Delete</Button>
                    <Button variant={"light"} color={"danger"} onPress={() => props.onClose(false)}>Cancel</Button>
                </ModalFooter>
            </ModalContent>
        </Modal>
    );
}