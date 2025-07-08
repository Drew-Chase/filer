import {Button, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader} from "@heroui/react";

type SettingsProperties = {
    isOpen: boolean;
    onClose: () => void;
};

export default function SettingsModal(props: SettingsProperties)
{
    return (
        <Modal isOpen={props.isOpen} onClose={props.onClose}>
            <ModalContent>
                {onClose => (
                    <>
                        <ModalHeader>Settings</ModalHeader>
                        <ModalBody>
                            No Body
                        </ModalBody>
                        <ModalFooter>
                            <Button onPress={onClose}>Close</Button>
                        </ModalFooter>
                    </>
                )}
            </ModalContent>
        </Modal>
    );
}