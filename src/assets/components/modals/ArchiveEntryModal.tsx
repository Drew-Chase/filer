import {FileSystem} from "../../ts/filesystem.ts";

import {Button, Chip, Modal, ModalBody, ModalContent, ModalFooter, ModalHeader, Progress} from "@heroui/react";
import {useFileSystemEntry} from "../../providers/FileSystemEntryProvider.tsx";
import {useState} from "react";
import {motion} from "framer-motion";
import Input from "../overrides/Input.tsx";

type ArchiveEntryProperties = {
    isOpen: boolean;
    onClose: () => void;
};

export default function ArchiveEntryModal(props: ArchiveEntryProperties)
{
    const {selectedEntries, currentPath, refresh} = useFileSystemEntry();
    const [input, setInput] = useState("");
    const [isProcessing, setIsProcessing] = useState(false);
    const [progress, setProgress] = useState(0);
    const [errorMessage, setErrorMessage] = useState<string | null>(null);
    return (
        <Modal
            isOpen={props.isOpen}
            onClose={props.onClose}
            scrollBehavior={"inside"}
            backdrop={"blur"}
            classNames={{
                base: "bg-gradient-to-tr from-[#1d0a3b] to-[#2f115c]"
            }}
        >
            <ModalContent>
                {onClose => (
                    <>
                        <ModalHeader>Create Zip Archive</ModalHeader>
                        <ModalBody className={"overflow-hidden"}>
                            <motion.div
                                className={"w-full"}
                                initial={{opacity: 1, height: "unset"}}
                                animate={{opacity: isProcessing ? 0 : 1, height: isProcessing ? 0 : "unset"}}
                                transition={{duration: 0.5, delay: 0, type: "spring", ease: "easeInOut"}}
                            >
                                <Input
                                    label={"Archive Name"}
                                    value={input}
                                    onValueChange={setInput}
                                    description={"The name of the archive file, this will be appended with .zip"}
                                    endContent={<Chip className={"bg-white/5"}>.zip</Chip>}
                                />
                            </motion.div>
                            <motion.div
                                className={"w-full"}
                                initial={{opacity: 0, height: 0}}
                                animate={{opacity: isProcessing ? 1 : 0, height: isProcessing ? "unset" : 0}}
                                transition={{duration: 0.5, delay: 0, type: "spring", ease: "easeInOut"}}
                            >
                                <Progress
                                    label={`archiving ${selectedEntries.size} files...`}
                                    value={progress}
                                    minValue={0}
                                    maxValue={100}
                                    color={"primary"}
                                />
                            </motion.div>
                            {errorMessage && <div className={"text-danger"}>{errorMessage}</div>}
                        </ModalBody>
                        <ModalFooter>
                            <Button
                                isLoading={isProcessing}
                                color={"secondary"}
                                onPress={async () =>
                                {
                                    setIsProcessing(true);
                                    FileSystem.archive(
                                        `${input}.zip`,
                                        [...selectedEntries].map(i => i.filename),
                                        currentPath!,
                                        setProgress,
                                        () =>
                                        {
                                            refresh();
                                            onClose();
                                            setIsProcessing(false);
                                            setErrorMessage(null);
                                            setInput("");
                                            setProgress(0);
                                        },
                                        (msg) =>
                                        {
                                            setIsProcessing(false);
                                            setErrorMessage(msg);
                                        }
                                    );
                                }}
                            >
                                Archive
                            </Button>
                            <Button onPress={onClose} color={"danger"} variant={"light"} isLoading={isProcessing}>Cancel</Button>
                        </ModalFooter>
                    </>
                )}
            </ModalContent>
        </Modal>
    );
}