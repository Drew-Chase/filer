import {Modal, ModalBody, ModalContent, ModalHeader} from "@heroui/react";
import DragDropArea from "../upload/DragDropArea.tsx";
import {useState} from "react";
import FileUploadProgress from "../upload/FileUploadProgress.tsx";

type UploadEntryProperties = {
    isOpen: boolean;
    onClose: () => void;
};

export default function UploadEntryModal(props: UploadEntryProperties)
{
    const [uploadFiles, setUploadFiles] = useState<File[]>([]);
    return (
        <Modal
            isOpen={props.isOpen}
            onClose={() =>
            {
                setUploadFiles([]);
                props.onClose();
            }}
            size={"3xl"}
            backdrop={"blur"}
            hideCloseButton={uploadFiles.length > 0}
            isDismissable={uploadFiles.length === 0}
            isKeyboardDismissDisabled={uploadFiles.length === 0}
            scrollBehavior={"inside"}
            classNames={{
                base: "bg-gradient-to-tr from-[#1d0a3b] to-[#2f115c]"
            }}
        >
            <ModalContent>
                <ModalHeader>Upload Files</ModalHeader>
                <ModalBody>
                    {uploadFiles.length === 0 ?
                        <DragDropArea onFileSelected={setUploadFiles}/> :
                        <FileUploadProgress files={uploadFiles}/>
                    }
                </ModalBody>
            </ModalContent>
        </Modal>
    );
}