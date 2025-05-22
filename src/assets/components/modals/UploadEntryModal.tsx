import {Modal, ModalBody, ModalContent, ModalHeader} from "@heroui/react";
import DragDropArea from "../upload/DragDropArea.tsx";
import {useState} from "react";
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
    const {refresh} = useFileSystemEntry();
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
                            onUploadComplete={() =>
                            {
                                refresh();
                                setUploadFiles([]);
                                props.onClose();
                            }}
                        />
                    }
                </ModalBody>
            </ModalContent>
        </Modal>
    );
}