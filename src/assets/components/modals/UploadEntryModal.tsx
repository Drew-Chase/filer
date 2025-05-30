import {Modal, ModalBody, ModalContent, ModalHeader, ModalFooter, Button} from "@heroui/react";
import DragDropArea from "../upload/DragDropArea.tsx";
import {useState, useRef} from "react";
import FileUploadProgress from "../upload/FileUploadProgress.tsx";
import {useFileSystemEntry} from "../../providers/FileSystemEntryProvider.tsx";
import {FileSystem} from "../../ts/filesystem.ts";

type UploadEntryProperties = {
    isOpen: boolean;
    onClose: () => void;
};

export default function UploadEntryModal(props: UploadEntryProperties)
{
    const [uploadFiles, setUploadFiles] = useState<File[]>([]);
    const [uploadSpeed, setUploadSpeed] = useState(0);
    const [isUploading, setIsUploading] = useState(false);
    const {refresh} = useFileSystemEntry();
    const uploadCancelFunctionsRef = useRef<Map<File, () => Promise<void>>>(new Map());

    const handleCancelAll = async () => {
        const cancelPromises: Promise<void>[] = [];
        uploadCancelFunctionsRef.current.forEach(cancelFn => {
            cancelPromises.push(cancelFn());
        });

        try {
            await Promise.all(cancelPromises);
            // Clear the map after all cancellations are processed
            uploadCancelFunctionsRef.current.clear();
        } catch (error) {
            console.error("Error cancelling uploads:", error);
        }
    };

    const registerCancelFunction = (file: File, cancelFn: () => Promise<void>) => {
        uploadCancelFunctionsRef.current.set(file, cancelFn);
        setIsUploading(true);
    };

    return (
        <Modal
            isOpen={props.isOpen}
            onClose={() =>
            {
                if (isUploading) {
                    // If uploads are in progress, don't close the modal
                    return;
                }
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
                <ModalHeader className={"flex flex-row items-center"}>
                    <div className={"w-full"}>Upload Files</div>
                    {(uploadFiles.length > 0 && uploadSpeed !== 0 && !isNaN(uploadSpeed)) ? (<div className={"w-full text-sm text-nowrap text-end"}>{FileSystem.formatSize(uploadSpeed)}/s</div>) : (<></>)}
                </ModalHeader>
                <ModalBody>
                    {uploadFiles.length === 0 ?
                        <DragDropArea onFileSelected={setUploadFiles}/> :
                        <FileUploadProgress
                            onUploadSpeedChange={setUploadSpeed}
                            files={uploadFiles}
                            registerCancelFunction={registerCancelFunction}
                            onUploadComplete={() =>
                            {
                                refresh();
                                setUploadFiles([]);
                                setIsUploading(false);
                                uploadCancelFunctionsRef.current.clear();
                                props.onClose();
                            }}
                        />
                    }
                </ModalBody>
                {uploadFiles.length > 0 && (
                    <ModalFooter>
                        <Button 
                            color="danger" 
                            variant="light" 
                            onPress={async () => {
                                if (isUploading) {
                                    // If uploads are in progress, cancel them
                                    await handleCancelAll();
                                } else {
                                    // Otherwise just close the modal
                                    setUploadFiles([]);
                                    props.onClose();
                                }
                            }}
                        >
                            {isUploading ? "Cancel Uploads" : "Close"}
                        </Button>
                    </ModalFooter>
                )}
            </ModalContent>
        </Modal>
    );
}
